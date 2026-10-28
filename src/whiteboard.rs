use iced::{mouse, touch};
use iced::widget::canvas::{self, Frame, Geometry, LineCap, LineJoin, Path, Program, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteboardTool {
    Pen,
    Highlighter,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperPattern {
    DotGrid,
    Ruled,
    MathGrid,
    Blank,
}

impl PaperPattern {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DotGrid => "Dots",
            Self::Ruled => "Ruled",
            Self::MathGrid => "Math Grid",
            Self::Blank => "Blank",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrokePath {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
    pub is_highlighter: bool,
}

#[derive(Debug, Clone)]
pub struct WhiteboardState {
    pub strokes: Vec<StrokePath>,
    pub undo_stack: Vec<Vec<StrokePath>>,
    pub redo_stack: Vec<Vec<StrokePath>>,
    pub current_stroke: Option<StrokePath>,
    pub active_tool: WhiteboardTool,
    pub pen_color: Color,
    pub pen_width: f32,
    pub paper_pattern: PaperPattern,
    pub is_drawing: bool,
}

impl Default for WhiteboardState {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_stroke: None,
            active_tool: WhiteboardTool::Pen,
            pen_color: Color::from_rgb(0.85, 0.88, 0.92), // Off-white
            pen_width: 2.5,
            paper_pattern: PaperPattern::DotGrid,
            is_drawing: false,
        }
    }
}

impl WhiteboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_stroke(&mut self, point: Point) {
        self.is_drawing = true;
        self.redo_stack.clear();

        match self.active_tool {
            WhiteboardTool::Pen => {
                self.current_stroke = Some(StrokePath {
                    points: vec![point],
                    color: self.pen_color,
                    width: self.pen_width,
                    is_highlighter: false,
                });
            }
            WhiteboardTool::Highlighter => {
                let mut hl_color = self.pen_color;
                hl_color.a = 0.35;
                self.current_stroke = Some(StrokePath {
                    points: vec![point],
                    color: hl_color,
                    width: 16.0,
                    is_highlighter: true,
                });
            }
            WhiteboardTool::Eraser => {
                self.erase_at(point, 18.0);
            }
        }
    }

    pub fn continue_stroke(&mut self, point: Point) {
        if !self.is_drawing {
            return;
        }

        match self.active_tool {
            WhiteboardTool::Pen | WhiteboardTool::Highlighter => {
                if let Some(ref mut stroke) = self.current_stroke {
                    if let Some(last) = stroke.points.last() {
                        let dx = point.x - last.x;
                        let dy = point.y - last.y;
                        if (dx * dx + dy * dy) >= 2.0 {
                            stroke.points.push(point);
                        }
                    } else {
                        stroke.points.push(point);
                    }
                }
            }
            WhiteboardTool::Eraser => {
                self.erase_at(point, 18.0);
            }
        }
    }

    pub fn end_stroke(&mut self) {
        if !self.is_drawing {
            return;
        }
        self.is_drawing = false;

        if let Some(stroke) = self.current_stroke.take() {
            if !stroke.points.is_empty() {
                self.undo_stack.push(self.strokes.clone());
                self.strokes.push(stroke);
            }
        }
    }

    pub fn erase_at(&mut self, center: Point, radius: f32) {
        let r2 = radius * radius;
        let prev_len = self.strokes.len();

        let mut strokes_to_keep = Vec::new();
        let mut any_erased = false;

        for stroke in &self.strokes {
            let hit = stroke.points.iter().any(|p| {
                let dx = p.x - center.x;
                let dy = p.y - center.y;
                (dx * dx + dy * dy) <= r2
            });

            if !hit {
                strokes_to_keep.push(stroke.clone());
            } else {
                any_erased = true;
            }
        }

        if any_erased && strokes_to_keep.len() != prev_len {
            self.undo_stack.push(self.strokes.clone());
            self.strokes = strokes_to_keep;
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.strokes.clone());
            self.strokes = prev;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.strokes.clone());
            self.strokes = next;
        }
    }

    /// Instant wipe for students when calculations / notes are completed
    pub fn clear_board(&mut self) {
        if !self.strokes.is_empty() {
            self.undo_stack.push(self.strokes.clone());
            self.redo_stack.clear();
            self.strokes.clear();
        }
    }
}

#[derive(Debug, Clone)]
pub enum WhiteboardMessage {
    PointPressed(Point),
    PointDragged(Point),
    PointReleased,
}

pub struct WhiteboardCanvas<'a> {
    pub state: &'a WhiteboardState,
}

impl<'a, Message> Program<Message, Theme, Renderer> for WhiteboardCanvas<'a>
where
    Message: 'static + From<WhiteboardMessage>,
{
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            // 1. Touch Events (Pen Tablet Stylus & Touchscreen)
            canvas::Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                let rel_x = position.x - bounds.x;
                let rel_y = position.y - bounds.y;
                if rel_x >= 0.0 && rel_x <= bounds.width && rel_y >= 0.0 && rel_y <= bounds.height {
                    (
                        canvas::event::Status::Captured,
                        Some(Message::from(WhiteboardMessage::PointPressed(Point::new(rel_x, rel_y)))),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                if self.state.is_drawing {
                    let rel_x = (position.x - bounds.x).clamp(0.0, bounds.width);
                    let rel_y = (position.y - bounds.y).clamp(0.0, bounds.height);
                    (
                        canvas::event::Status::Captured,
                        Some(Message::from(WhiteboardMessage::PointDragged(Point::new(rel_x, rel_y)))),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Touch(touch::Event::FingerLifted { .. })
            | canvas::Event::Touch(touch::Event::FingerLost { .. }) => {
                if self.state.is_drawing {
                    (
                        canvas::event::Status::Captured,
                        Some(Message::from(WhiteboardMessage::PointReleased)),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }

            // 2. Mouse Events (Standard Mouse & Pen Stylus Left Click)
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(cursor_pos) = cursor.position_in(bounds) {
                    (
                        canvas::event::Status::Captured,
                        Some(Message::from(WhiteboardMessage::PointPressed(cursor_pos))),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if self.state.is_drawing {
                    let pos = if let Some(p) = cursor.position_in(bounds) {
                        p
                    } else if let Some(p) = cursor.position() {
                        Point::new(
                            (p.x - bounds.x).clamp(0.0, bounds.width),
                            (p.y - bounds.y).clamp(0.0, bounds.height),
                        )
                    } else {
                        return (canvas::event::Status::Ignored, None);
                    };
                    (
                        canvas::event::Status::Captured,
                        Some(Message::from(WhiteboardMessage::PointDragged(pos))),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                if self.state.is_drawing {
                    (
                        canvas::event::Status::Captured,
                        Some(Message::from(WhiteboardMessage::PointReleased)),
                    )
                } else {
                    (canvas::event::Status::Ignored, None)
                }
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // 1. Draw Paper Pattern
        let width = bounds.width;
        let height = bounds.height;

        match self.state.paper_pattern {
            PaperPattern::DotGrid => {
                let dot_spacing = 24.0;
                let dot_color = Color::from_rgba(0.25, 0.28, 0.35, 0.30);
                let mut x = dot_spacing;
                while x < width {
                    let mut y = dot_spacing;
                    while y < height {
                        let dot = Path::circle(Point::new(x, y), 0.9);
                        frame.fill(&dot, dot_color);
                        y += dot_spacing;
                    }
                    x += dot_spacing;
                }
            }
            PaperPattern::Ruled => {
                let line_spacing = 28.0;
                let line_color = Color::from_rgba(0.22, 0.25, 0.32, 0.35);
                let mut y = line_spacing;
                while y < height {
                    let p1 = Point::new(0.0, y);
                    let p2 = Point::new(width, y);
                    let line = Path::line(p1, p2);
                    frame.stroke(
                        &line,
                        Stroke::default()
                            .with_color(line_color)
                            .with_width(1.0),
                    );
                    y += line_spacing;
                }
            }
            PaperPattern::MathGrid => {
                let grid_size = 20.0;
                let grid_color = Color::from_rgba(0.20, 0.23, 0.30, 0.30);

                let mut x = grid_size;
                while x < width {
                    let p1 = Point::new(x, 0.0);
                    let p2 = Point::new(x, height);
                    frame.stroke(
                        &Path::line(p1, p2),
                        Stroke::default().with_color(grid_color).with_width(0.8),
                    );
                    x += grid_size;
                }

                let mut y = grid_size;
                while y < height {
                    let p1 = Point::new(0.0, y);
                    let p2 = Point::new(width, y);
                    frame.stroke(
                        &Path::line(p1, p2),
                        Stroke::default().with_color(grid_color).with_width(0.8),
                    );
                    y += grid_size;
                }
            }
            PaperPattern::Blank => {
                // Pure clean dark slate, no grid lines
            }
        }

        // 2. Render Completed Strokes
        for stroke in &self.state.strokes {
            draw_stroke(&mut frame, stroke);
        }

        // 3. Render Active In-Progress Stroke
        if let Some(ref current) = self.state.current_stroke {
            draw_stroke(&mut frame, current);
        }

        // 4. Render Eraser Reticle
        if self.state.active_tool == WhiteboardTool::Eraser {
            if let Some(pos) = cursor.position_in(bounds) {
                let circle = Path::circle(pos, 18.0);
                frame.stroke(
                    &circle,
                    Stroke::default()
                        .with_color(Color::from_rgba(0.9, 0.4, 0.4, 0.7))
                        .with_width(1.5),
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

fn draw_stroke(frame: &mut Frame, stroke: &StrokePath) {
    if stroke.points.is_empty() {
        return;
    }

    if stroke.points.len() == 1 {
        let p = stroke.points[0];
        let dot = Path::circle(p, stroke.width / 2.0);
        frame.fill(&dot, stroke.color);
        return;
    }

    let path = Path::new(|builder| {
        let (first, rest) = stroke.points.split_first().unwrap();
        builder.move_to(*first);
        for pt in rest {
            builder.line_to(*pt);
        }
    });

    let stroke_style = Stroke::default()
        .with_color(stroke.color)
        .with_width(stroke.width)
        .with_line_cap(LineCap::Round)
        .with_line_join(LineJoin::Round);

    frame.stroke(&path, stroke_style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whiteboard_stroke_lifecycle() {
        let mut board = WhiteboardState::new();
        assert_eq!(board.strokes.len(), 0);

        board.start_stroke(Point::new(10.0, 10.0));
        board.continue_stroke(Point::new(15.0, 15.0));
        board.continue_stroke(Point::new(20.0, 20.0));
        board.end_stroke();

        assert_eq!(board.strokes.len(), 1);
        assert_eq!(board.strokes[0].points.len(), 3);

        // Wipe board for students
        board.clear_board();
        assert_eq!(board.strokes.len(), 0);

        // Can undo wipe
        board.undo();
        assert_eq!(board.strokes.len(), 1);
    }

    #[test]
    fn test_whiteboard_eraser() {
        let mut board = WhiteboardState::new();
        board.start_stroke(Point::new(50.0, 50.0));
        board.continue_stroke(Point::new(55.0, 55.0));
        board.end_stroke();
        assert_eq!(board.strokes.len(), 1);

        // Erase away from stroke (no hit)
        board.erase_at(Point::new(10.0, 10.0), 10.0);
        assert_eq!(board.strokes.len(), 1);

        // Erase on stroke (hit)
        board.erase_at(Point::new(52.0, 52.0), 10.0);
        assert_eq!(board.strokes.len(), 0);
    }
}
