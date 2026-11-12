use cosmic::iced::core::layout;
use cosmic::iced::core::mouse;
use cosmic::iced::core::overlay;
use cosmic::iced::core::renderer;
use cosmic::iced::core::widget::{Operation, Tree};
use cosmic::iced::core::{Clipboard, Event, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget};
use cosmic::{Element, Renderer, Theme};

/// Lays its children out one after another, like a row or column, but stretches each of them across the strip to
/// the thickness of the thickest one. So each indicator, and what reports it as hovered, spans the whole applet
/// across the panel rather than only its own text.
///
/// Children that are a fixed thickness across the strip keep it, and are centred.
pub struct IndicatorStrip<'a, Message> {
    children: Vec<Element<'a, Message>>,
    axis: Axis,
    spacing: f32,
}

pub fn indicator_strip<'a, Message>(
    horizontal: bool,
    spacing: impl Into<f32>,
    children: Vec<Element<'a, Message>>,
) -> IndicatorStrip<'a, Message> {
    IndicatorStrip {
        children,
        axis: Axis { horizontal },
        spacing: spacing.into(),
    }
}

/// The direction the strip runs in, to work with lengths along and across it.
#[derive(Clone, Copy)]
struct Axis {
    horizontal: bool,
}

impl Axis {
    /// Packs lengths along and across the strip into a size.
    fn pack(self, along: f32, across: f32) -> Size {
        if self.horizontal {
            Size::new(along, across)
        } else {
            Size::new(across, along)
        }
    }

    fn along<T>(self, size: Size<T>) -> T {
        if self.horizontal { size.width } else { size.height }
    }

    fn across<T>(self, size: Size<T>) -> T {
        if self.horizontal { size.height } else { size.width }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for IndicatorStrip<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(&mut self.children);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(Length::Shrink).height(Length::Shrink);
        let axis = self.axis;
        let max = limits.max();
        let max_along = axis.along(max);
        let max_across = axis.across(max);
        // Fluid lengths along the strip shrink to their content, as they do in a shrinking row or column.
        let compression = Size::new(axis.horizontal, !axis.horizontal);
        let child_limits =
            |min: Size, max: Size| layout::Limits::with_compression(min, max, compression);

        // The thickness of the strip is that of its thickest child, leaving out the fluid ones which would take all
        // the room they're given.
        let mut thickness: f32 = 0.0;
        for (child, tree) in self.children.iter_mut().zip(tree.children.iter_mut()) {
            if axis.across(child.as_widget().size()).fill_factor() == 0 {
                let limits = child_limits(Size::ZERO, axis.pack(max_along, max_across));
                let node = child.as_widget_mut().layout(tree, renderer, &limits);
                thickness = thickness.max(axis.across(node.size()));
            }
        }

        let spacing = self.spacing;
        let mut along: f32 = 0.0;
        let mut nodes = Vec::with_capacity(self.children.len());
        for (index, (child, tree)) in self.children.iter_mut().zip(tree.children.iter_mut()).enumerate() {
            if index > 0 {
                along += spacing;
            }
            let min_across = match axis.across(child.as_widget().size()) {
                Length::Fixed(_) => 0.0,
                _ => thickness,
            };
            let limits = child_limits(
                axis.pack(0.0, min_across),
                axis.pack((max_along - along).max(0.0), thickness),
            );
            let node = child.as_widget_mut().layout(tree, renderer, &limits);
            let offset = ((thickness - axis.across(node.size())) / 2.0).max(0.0);
            let position = axis.pack(along, offset);
            along += axis.along(node.size());
            nodes.push(node.move_to(Point::new(position.width, position.height)));
        }

        layout::Node::with_children(axis.pack(along, thickness), nodes)
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((child, tree), layout) in self.children.iter_mut().zip(&mut tree.children).zip(layout.children()) {
                child.as_widget_mut().operate(tree, layout, renderer, operation);
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self.children.iter_mut().zip(&mut tree.children).zip(layout.children()) {
            child
                .as_widget_mut()
                .update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self.children.iter().zip(&tree.children).zip(layout.children()) {
            child
                .as_widget()
                .draw(tree, renderer, theme, style, layout, cursor, viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(&mut self.children, tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message: 'a> From<IndicatorStrip<'a, Message>> for Element<'a, Message> {
    fn from(strip: IndicatorStrip<'a, Message>) -> Self {
        Element::new(strip)
    }
}
