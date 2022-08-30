use {
    crate::{
        ast,
        ast::Quant,
        program,
        program::{Instr, InstrPtr},
        Ast, CharClass, Program, Range, Utf8Encoder,
    },
    std::collections::HashMap,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct CodeGenerator {
    utf8_encoder: Utf8Encoder,
    states: Vec<State>,
    instr_cache: HashMap<Instr, InstrPtr>,
}

impl CodeGenerator {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn generate(&mut self, ast: &Ast, options: Options) -> Program {
        CompileContext {
            encoder: &mut self.utf8_encoder,
            states: &mut self.states,
            instr_cache: &mut self.instr_cache,
            options,
            slot_count: 0,
            instrs: Vec::new(),
        }
        .generate(ast)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Options {
    pub(crate) reverse: bool,
    pub(crate) bytes: bool,
    pub(crate) dot_star: bool,
}

#[derive(Debug)]
struct CompileContext<'a> {
    encoder: &'a mut Utf8Encoder,
    states: &'a mut Vec<State>,
    instr_cache: &'a mut HashMap<Instr, InstrPtr>,
    options: Options,
    slot_count: usize,
    instrs: Vec<Instr>,
}

impl<'a> CompileContext<'a> {
    fn generate(mut self, ast: &Ast) -> Program {
        let mut frag = self.generate_recursive(ast);
        frag = self.generate_cap(frag, 0);
        self.options.reverse = false;
        let match_frag = self.generate_match();
        frag = self.generate_cat(frag, match_frag);
        if self.options.dot_star {
            let dot_star_frag = self.generate_char_class(&CharClass::any());
            let dot_star_frag = self.generate_star(dot_star_frag, true);
            frag = self.generate_cat(dot_star_frag, frag);
        }
        Program {
            slot_count: self.slot_count,
            instrs: self.instrs,
            start: frag.start,
        }
    }

    fn generate_recursive(&mut self, ast: &Ast) -> Frag {
        match *ast {
            Ast::Char(ch) => self.generate_char(ch),
            Ast::CharClass(ref char_class) => self.generate_char_class(char_class),
            Ast::Cap(ref ast, index) => {
                let frag = self.generate_recursive(ast);
                self.generate_cap(frag, index)
            }
            Ast::Assert(pred) => self.generate_assert(pred),
            Ast::Rep(ref ast, Quant::Quest(is_lazy)) => {
                let frag = self.generate_recursive(ast);
                self.generate_quest(frag, is_lazy)
            }
            Ast::Rep(ref ast, Quant::Star(is_lazy)) => {
                let frag = self.generate_recursive(ast);
                self.generate_star(frag, is_lazy)
            }
            Ast::Rep(ref ast, Quant::Plus(is_lazy)) => {
                let frag = self.generate_recursive(ast);
                self.generate_plus(frag, is_lazy)
            }
            Ast::Cat(ref asts) => {
                let mut asts = asts.iter();
                let mut acc_frag = self.generate_recursive(asts.next().unwrap());
                for ast in asts {
                    let frag = self.generate_recursive(ast);
                    acc_frag = self.generate_cat(acc_frag, frag);
                }
                acc_frag
            }
            Ast::Alt(ref asts) => {
                let mut asts = asts.iter();
                let mut acc_frag = self.generate_recursive(asts.next().unwrap());
                for ast in asts {
                    let frag = self.generate_recursive(ast);
                    acc_frag = self.generate_alt(acc_frag, frag);
                }
                acc_frag
            }
        }
    }

    fn generate_match(&mut self) -> Frag {
        Frag {
            start: self.emit_instr(Instr::Match),
            ends: HolePtrList::new(),
        }
    }

    fn generate_byte_range(&mut self, byte_range: Range<u8>) -> Frag {
        let instr = self.emit_instr(Instr::ByteRange(byte_range, program::NULL_INSTR_PTR));
        Frag {
            start: instr,
            ends: HolePtrList::unit(HolePtr::next_0(instr)),
        }
    }

    fn generate_char(&mut self, ch: char) -> Frag {
        if self.options.bytes {
            let mut bytes = [0; 4];
            let mut bytes = ch.encode_utf8(&mut bytes).bytes();
            let byte = bytes.next().unwrap();
            let mut acc_frag = self.generate_byte_range(Range::new(byte, byte));
            for byte in bytes {
                let frag = self.generate_byte_range(Range::new(byte, byte));
                acc_frag = self.generate_cat(acc_frag, frag);
            }
            acc_frag
        } else {
            let instr = self.emit_instr(Instr::Char(ch, program::NULL_INSTR_PTR));
            Frag {
                start: instr,
                ends: HolePtrList::unit(HolePtr::next_0(instr)),
            }
        }
    }

    fn generate_char_class(&mut self, char_class: &CharClass) -> Frag {
        if self.options.bytes {
            let mut suffix_tree = SuffixTree {
                states: self.states,
                suffix_cache: SuffixCache {
                    instr_cache: self.instr_cache,
                    instrs: &mut self.instrs,
                },
                options: self.options,
                ends: HolePtrList::new(),
            };
            if self.options.reverse {
                for char_range in char_class {
                    for byte_ranges in self.encoder.encode(char_range) {
                        suffix_tree.add_byte_ranges(&byte_ranges);
                    }
                }
            } else {
                for char_range in char_class {
                    for mut byte_ranges in self.encoder.encode(char_range) {
                        byte_ranges.reverse();
                        suffix_tree.add_byte_ranges(&byte_ranges);
                    }
                }
            }
            suffix_tree.generate()
        } else {
            let instr = self.emit_instr(Instr::CharClass(
                char_class.clone(),
                program::NULL_INSTR_PTR,
            ));
            Frag {
                start: instr,
                ends: HolePtrList::unit(HolePtr::next_0(instr)),
            }
        }
    }

    fn generate_cap(&mut self, frag: Frag, cap_index: usize) -> Frag {
        let first_slot_index = cap_index * 2;
        self.slot_count = self.slot_count.max(first_slot_index + 2);
        let instr_0 = self.emit_instr(Instr::Save(first_slot_index, frag.start));
        let instr_1 = self.emit_instr(Instr::Save(first_slot_index + 1, program::NULL_INSTR_PTR));
        frag.ends.fill(instr_1, &mut self.instrs);
        Frag {
            start: instr_0,
            ends: HolePtrList::unit(HolePtr::next_0(instr_1)),
        }
    }

    fn generate_assert(&mut self, pred: ast::Pred) -> Frag {
        let instr = self.emit_instr(Instr::Assert(
            if self.options.reverse {
                match pred {
                    ast::Pred::IsAtStartOfText => program::Pred::IsAtEndOfText,
                    ast::Pred::IsAtEndOfText => program::Pred::IsAtStartOfText,
                }
            } else {
                match pred {
                    ast::Pred::IsAtStartOfText => program::Pred::IsAtStartOfText,
                    ast::Pred::IsAtEndOfText => program::Pred::IsAtEndOfText,
                }
            },
            program::NULL_INSTR_PTR,
        ));
        Frag {
            start: instr,
            ends: HolePtrList::unit(HolePtr::next_0(instr)),
        }
    }

    fn generate_quest(&mut self, frag: Frag, is_lazy: bool) -> Frag {
        let instr;
        let hole;
        if is_lazy {
            instr = self.emit_instr(Instr::Split(program::NULL_INSTR_PTR, frag.start));
            hole = HolePtr::next_0(instr);
        } else {
            instr = self.emit_instr(Instr::Split(frag.start, program::NULL_INSTR_PTR));
            hole = HolePtr::next_1(instr);
        }
        Frag {
            start: instr,
            ends: frag.ends.append(hole, &mut self.instrs),
        }
    }

    fn generate_star(&mut self, frag: Frag, is_lazy: bool) -> Frag {
        let instr;
        let hole;
        if is_lazy {
            instr = self.emit_instr(Instr::Split(program::NULL_INSTR_PTR, frag.start));
            hole = HolePtr::next_0(instr);
        } else {
            instr = self.emit_instr(Instr::Split(frag.start, program::NULL_INSTR_PTR));
            hole = HolePtr::next_1(instr);
        }
        frag.ends.fill(instr, &mut self.instrs);
        Frag {
            start: instr,
            ends: HolePtrList::unit(hole),
        }
    }

    fn generate_plus(&mut self, frag: Frag, is_lazy: bool) -> Frag {
        let instr;
        let hole;
        if is_lazy {
            instr = self.emit_instr(Instr::Split(program::NULL_INSTR_PTR, frag.start));
            hole = HolePtr::next_0(instr);
        } else {
            instr = self.emit_instr(Instr::Split(frag.start, program::NULL_INSTR_PTR));
            hole = HolePtr::next_1(instr);
        }
        frag.ends.fill(instr, &mut self.instrs);
        Frag {
            start: frag.start,
            ends: HolePtrList::unit(hole),
        }
    }

    fn generate_cat(&mut self, mut frag_0: Frag, mut frag_1: Frag) -> Frag {
        use std::mem;

        if self.options.reverse {
            mem::swap(&mut frag_0, &mut frag_1);
        }
        frag_0.ends.fill(frag_1.start, &mut self.instrs);
        Frag {
            start: frag_0.start,
            ends: frag_1.ends,
        }
    }

    fn generate_alt(&mut self, frag_0: Frag, frag_1: Frag) -> Frag {
        Frag {
            start: self.emit_instr(Instr::Split(frag_0.start, frag_1.start)),
            ends: frag_0.ends.concat(frag_1.ends, &mut self.instrs),
        }
    }

    fn emit_instr(&mut self, instr: Instr) -> InstrPtr {
        let instr_ptr = self.instrs.len();
        self.instrs.push(instr);
        instr_ptr
    }
}

#[derive(Debug)]
struct SuffixTree<'a> {
    states: &'a mut Vec<State>,
    suffix_cache: SuffixCache<'a>,
    options: Options,
    ends: HolePtrList,
}

impl<'a> SuffixTree<'a> {
    fn generate(mut self) -> Frag {
        let start = self.generate_suffix(0);
        self.suffix_cache.instr_cache.clear();
        if start == program::NULL_INSTR_PTR {
            let instr = self
                .suffix_cache
                .emit_instr(Instr::Nop(program::NULL_INSTR_PTR));
            Frag {
                start: instr,
                ends: HolePtrList::unit(HolePtr::next_0(instr)),
            }
        } else {
            Frag {
                start,
                ends: self.ends,
            }
        }
    }

    fn add_byte_ranges(&mut self, byte_ranges: &[Range<u8>]) {
        let index = self.prefix_len(byte_ranges);
        let instr = self.generate_suffix(index);
        self.extend_suffix(instr, &byte_ranges[index..]);
    }

    fn prefix_len(&self, byte_ranges: &[Range<u8>]) -> usize {
        if self.options.reverse {
            0
        } else {
            byte_ranges
                .iter()
                .zip(self.states.iter())
                .take_while(|&(&byte_range, state)| byte_range == state.byte_range)
                .count()
        }
    }

    fn generate_suffix(&mut self, start: usize) -> InstrPtr {
        use std::mem;

        let mut acc_instr = program::NULL_INSTR_PTR;
        for state in self.states.drain(start..).rev() {
            let has_hole = acc_instr == program::NULL_INSTR_PTR;
            let (instr, is_new) = self
                .suffix_cache
                .get_or_emit_instr(Instr::ByteRange(state.byte_range, acc_instr));
            acc_instr = instr;
            if is_new && has_hole {
                let ends = mem::replace(&mut self.ends, HolePtrList::new());
                self.ends = ends.append(HolePtr::next_0(instr), &mut self.suffix_cache.instrs);
            }
            if state.instr != program::NULL_INSTR_PTR {
                let (instr, _) = self
                    .suffix_cache
                    .get_or_emit_instr(Instr::Split(state.instr, acc_instr));
                acc_instr = instr;
            }
        }
        acc_instr
    }

    fn extend_suffix(&mut self, generated_instr: InstrPtr, byte_ranges: &[Range<u8>]) {
        let mut byte_ranges = byte_ranges.iter();
        self.states.push(State {
            instr: generated_instr,
            byte_range: *byte_ranges.next().unwrap(),
        });
        for &byte_range in byte_ranges {
            self.states.push(State {
                instr: program::NULL_INSTR_PTR,
                byte_range,
            });
        }
    }
}

#[derive(Debug)]
struct SuffixCache<'a> {
    instr_cache: &'a mut HashMap<Instr, InstrPtr>,
    instrs: &'a mut Vec<Instr>,
}

impl<'a> SuffixCache<'a> {
    fn get_or_emit_instr(&mut self, instr: Instr) -> (InstrPtr, bool) {
        match self.instr_cache.get(&instr) {
            Some(&ptr) => (ptr, false),
            None => {
                let ptr = self.emit_instr(instr.clone());
                self.instr_cache.insert(instr, ptr);
                (ptr, true)
            }
        }
    }

    fn emit_instr(&mut self, instr: Instr) -> InstrPtr {
        let instr_ptr = self.instrs.len();
        self.instrs.push(instr);
        instr_ptr
    }
}

#[derive(Clone, Debug)]
struct State {
    instr: InstrPtr,
    byte_range: Range<u8>,
}

#[derive(Debug)]
struct Frag {
    start: InstrPtr,
    ends: HolePtrList,
}

#[derive(Debug)]
struct HolePtrList {
    head: HolePtr,
    tail: HolePtr,
}

impl HolePtrList {
    fn new() -> Self {
        Self {
            head: HolePtr::null(),
            tail: HolePtr::null(),
        }
    }

    fn unit(hole: HolePtr) -> Self {
        Self {
            head: hole,
            tail: hole,
        }
    }

    fn append(self, hole: HolePtr, instrs: &mut [Instr]) -> Self {
        self.concat(Self::unit(hole), instrs)
    }

    fn concat(self, other: Self, instrs: &mut [Instr]) -> Self {
        if self.tail.is_null() {
            return other;
        }
        if other.head.is_null() {
            return self;
        }
        *self.tail.get_mut(instrs) = other.head.0;
        Self {
            head: self.head,
            tail: other.tail,
        }
    }

    fn fill(self, instr: InstrPtr, instrs: &mut [Instr]) {
        let mut current = self.head;
        while current.0 != program::NULL_INSTR_PTR {
            let next = *current.get(instrs);
            *current.get_mut(instrs) = instr;
            current = HolePtr(next);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HolePtr(usize);

impl HolePtr {
    fn null() -> Self {
        Self(program::NULL_INSTR_PTR)
    }

    fn next_0(instr: InstrPtr) -> Self {
        Self(instr << 1)
    }

    fn next_1(instr: InstrPtr) -> Self {
        Self(instr << 1 | 1)
    }

    fn is_null(self) -> bool {
        self.0 == program::NULL_INSTR_PTR
    }

    fn get(self, instrs: &[Instr]) -> &InstrPtr {
        let instr_ref = &instrs[self.0 >> 1];
        if self.0 & 1 == 0 {
            instr_ref.next_0()
        } else {
            instr_ref.next_1()
        }
    }

    fn get_mut(self, instrs: &mut [Instr]) -> &mut InstrPtr {
        let instr_ref = &mut instrs[self.0 >> 1];
        if self.0 & 1 == 0 {
            instr_ref.next_0_mut()
        } else {
            instr_ref.next_1_mut()
        }
    }
}
