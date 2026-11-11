use cosmic::iced::core::renderer::Quad;
use cosmic::iced::core::widget::{tree, Tree};
use cosmic::iced::core::{
    layout, mouse, renderer, Background, Border, Clipboard, Color, Event, Layout, Length, Point, Rectangle,
    Shadow, Shell, Size, Widget,
};
use cosmic::{Element, Renderer, Theme};

const CORNER_RADIUS: f32 = 4.0;
const HANDLE_RADIUS: f32 = 7.0;

/// A rectangle filled with stacked backgrounds, with a draggable handle. Reports the handle position as fractions of
/// the width and height, from the top left.
///
/// Every frame is a few quads, however the colours change, so dragging stays smooth with the software renderer.
pub struct GradientPad<'a, Message> {
    layers: Vec<Background>,
    handle: Point,
    handle_fill: Color,
    width: Length,
    height: Length,
    on_change: Box<dyn Fn(f32, f32) -> Message + 'a>,
}

impl<'a, Message> GradientPad<'a, Message> {
    /// `handle` is the position of the handle as fractions of the width and height.
    pub fn new(
        layers: Vec<Background>,
        handle: (f32, f32),
        handle_fill: Color,
        on_change: impl Fn(f32, f32) -> Message + 'a,
    ) -> Self {
        GradientPad {
            layers,
            handle: Point::new(handle.0, handle.1),
            handle_fill,
            width: Length::Fill,
            height: Length::Fixed(160.0),
            on_change: Box::new(on_change),
        }
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    fn publish(&self, bounds: Rectangle, position: Point, shell: &mut Shell<'_, Message>) {
        let x = ((position.x - bounds.x) / bounds.width).clamp(0.0, 1.0);
        let y = ((position.y - bounds.y) / bounds.height).clamp(0.0, 1.0);
        shell.publish((self.on_change)(x, y));
    }
}

#[derive(Default)]
struct State {
    dragging: bool,
}

impl<Message> Widget<Message, Theme, Renderer> for GradientPad<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        use cosmic::iced::core::Renderer as _;

        let bounds = layout.bounds();
        for layer in &self.layers {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border {
                        radius: CORNER_RADIUS.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                },
                *layer,
            );
        }

        // A white ring with a faint dark outline, so the handle shows on both light and dark colours.
        let centre = Point::new(
            bounds.x + self.handle.x * bounds.width,
            bounds.y + self.handle.y * bounds.height,
        );
        let circle = |radius: f32| Rectangle {
            x: centre.x - radius,
            y: centre.y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
        };
        renderer.fill_quad(
            Quad {
                bounds: circle(HANDLE_RADIUS + 1.0),
                border: Border {
                    radius: (HANDLE_RADIUS + 1.0).into(),
                    width: 1.0,
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                },
                shadow: Shadow::default(),
                ..Default::default()
            },
            Color::TRANSPARENT,
        );
        renderer.fill_quad(
            Quad {
                bounds: circle(HANDLE_RADIUS),
                border: Border {
                    radius: HANDLE_RADIUS.into(),
                    width: 2.0,
                    color: Color::WHITE,
                },
                shadow: Shadow::default(),
                ..Default::default()
            },
            self.handle_fill,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state: &mut State = tree.state.downcast_mut();
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(position) = cursor.position_over(bounds) {
                    state.dragging = true;
                    self.publish(bounds, position, shell);
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.dragging => {
                self.publish(bounds, *position, shell);
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left) | mouse::Event::CursorLeft)
                if state.dragging =>
            {
                state.dragging = false;
                shell.capture_event();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state: &State = tree.state.downcast_ref();
        if state.dragging || cursor.is_over(layout.bounds()) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: 'a> From<GradientPad<'a, Message>> for Element<'a, Message> {
    fn from(pad: GradientPad<'a, Message>) -> Self {
        Element::new(pad)
    }
}
