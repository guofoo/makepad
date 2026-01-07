# Accordion

The `Accordion` widget allows you to organize content into collapsible sections. It supports exclusive opening, meaning only one section can be open at a time within a group.

## Usage

```rust
<Accordion> {
    <AccordionItem> {
        header: <Button> { text: "Section 1" }
        body: <Label> { text: "Content for section 1" }
    }
    <AccordionItem> {
        header: <Button> { text: "Section 2" }
        body: <Label> { text: "Content for section 2" }
    }
}
```

## Properties

-   `header`: The widget to display as the header (always visible). Clicking it toggles the section.
-   `body`: The widget to display as the content (collapsible).
-   `opened`: The state of the section (0.0 to 1.0). Controlled by the animator.
-   `group`: An optional `LiveId` to group items for exclusive opening.

## Styling

You can style the `AccordionItem` container using `draw_bg`.

### Available Uniforms

| Uniform | Type | Description |
| :--- | :--- | :--- |
| `color` | `vec4` | Background color of the item. |
| `border_color` | `vec4` | Color of the border. |
| `border_width` | `float` | Width of the border. |
| `border_radius` | `float` | Radius of the corners. |

### Example

```rust
<AccordionItem> {
    draw_bg: {
        color: #f00
        border_color: #00f
        border_width: 1.0
        border_radius: 4.0
    }
}
```
