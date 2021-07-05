use crate::gl::*;
use cgmath::*;
use fxhash::*;
use std::mem;
use wasm_stopwatch::*;

use super::color::*;
use super::draw_2d::*;
use super::event::*;
use super::gui::*;

pub struct Label {
    id: WidgetId,
    text: String,
}

impl Label {
    pub fn new(text: &str) -> Box<Self> {
        Box::new(Label { id: WidgetId::new(), text: text.to_owned() })
    }
}

impl Widget for Label {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
        theme.font.draw_string(context, &self.text, rect.start, theme.label_color);
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        theme.font.string_size(context, &self.text)
    }
}

pub struct ButtonResult {
    pressed: bool,
}

impl ButtonResult {
    pub fn pressed(&self) -> bool {
        self.pressed
    }
}

#[derive(Clone)]
pub struct Button {
    id: WidgetId,
    text: String,
}

impl Button {
    pub fn new(text: &str) -> Box<Self> {
        let id = WidgetId::new();
        Box::new(Button { id, text: text.to_owned() })
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_owned();
    }
}

impl Component for Button {
    type Res = ButtonResult;

    fn update(&mut self, _theme: &Theme, events: Vec<Event>) -> ButtonResult {
        let mut pressed = false;
        for event in events {
            match event {
                Event::MouseDown(MouseButton::Left, _) => {
                    pressed = true;
                    break;
                }
                Event::KeyDown(key) => {
                    if key.code == "Enter" || key.code == "space" {
                        pressed = true;
                        break;
                    }
                }
                _ => (),
            }
        }

        ButtonResult { pressed }
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn is_component(&self) -> bool {
        true
    }

    fn draw(
        &self,
        context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        theme: &Theme,
        draw_2d: &mut Draw2d,
        cursor_pos: Option<Point2<i32>>,
        is_active: bool,
    ) {
        let fill_color =
            if cursor_pos.is_some() && rect.contains_point(cursor_pos.unwrap().cast().unwrap()) {
                theme.button_selected_fill_color
            } else if is_active {
                theme.button_active_fill_color
            } else {
                theme.button_fill_color
            };
        draw_2d.fill_rect(rect, fill_color);
        draw_2d.outline_rect(rect, theme.button_border_color, 1.0);
        theme.font.draw_string(
            context,
            &self.text,
            rect.start + vec2(2, 1),
            theme.button_text_color,
        );
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        theme.font.string_size(context, &self.text) + vec2(4, 2)
    }
}

/// A widget that makes its child its minimum possible size rather than filling the whole
/// window.
pub struct NoFill {
    id: WidgetId,
    child: Box<dyn Widget>,
}

impl NoFill {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(NoFill { id: WidgetId::new(), child })
    }
}

impl Widget for NoFill {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        min_sizes[&self.child.id()]
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&*self.child]
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FxHashMap<WidgetId, Rect<i32>>,
    ) {
        let min_size = min_sizes[&self.id()];
        widget_rects.insert(self.id(), Rect::new(rect.start, rect.start + min_size));
        self.child.compute_rects(
            Rect::new(rect.start, rect.start + min_size),
            theme,
            min_sizes,
            widget_rects,
        );
    }
}

pub struct Col {
    id: WidgetId,
    children: Vec<(Box<dyn Widget>, f32)>,
}

impl Col {
    pub fn new() -> Box<Self> {
        Box::new(Col { id: WidgetId::new(), children: vec![] })
    }

    /// Flex controls how to distribute unused space.
    pub fn child(mut self: Box<Self>, flex: f32, child: Box<dyn Widget>) -> Box<Self> {
        self.children.push((child, flex));
        self
    }

    pub fn children(mut self: Box<Self>, children: Vec<(f32, Box<dyn Widget>)>) -> Box<Self> {
        self.children.extend(children.into_iter().map(|(a, b)| (b, a)));
        self
    }
}

impl Widget for Col {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let mut min_size: Vector2<i32> = Vector2::zero();
        for &(ref child, _flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            min_size.x = min_size.x.max(child_min_size.x);
            min_size.y += child_min_size.y;
        }
        min_size
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|(child, _)| &**child as &dyn Widget).collect()
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FxHashMap<WidgetId, Rect<i32>>,
    ) {
        let total_flex = self.children.iter().map(|&(ref _child, flex)| flex).sum();
        let min_size = min_sizes[&self.id()];
        let own_rect = if total_flex == 0.0 {
            Rect::new(rect.start, rect.start + vec2(rect.size().x, min_size.y))
        } else {
            Rect::new(rect.start, rect.start + vec2(rect.size().x, rect.size().y))
        };
        widget_rects.insert(self.id(), own_rect);
        let mut next_pos = rect.start;
        let total_flex = if total_flex == 0.0 { 1.0 } else { total_flex };
        let extra_space = rect.size().y - min_size.y;
        for &(ref child, flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            let widget_extra_space = (extra_space as f32 * flex / total_flex) as i32;
            let widget_height = child_min_size.y + widget_extra_space;
            let widget_rect = Rect::new(next_pos, next_pos + vec2(rect.size().x, widget_height));
            next_pos.y += widget_height;
            child.compute_rects(widget_rect, theme, min_sizes, widget_rects);
        }
    }
}

pub struct Row {
    id: WidgetId,
    children: Vec<(Box<dyn Widget>, f32)>,
}

impl Row {
    pub fn new() -> Box<Self> {
        Box::new(Row { id: WidgetId::new(), children: vec![] })
    }

    /// Flex controls how to distribute unused space.
    pub fn child(mut self: Box<Self>, flex: f32, child: Box<dyn Widget>) -> Box<Self> {
        self.children.push((child, flex));
        self
    }

    pub fn children(mut self: Box<Self>, children: Vec<(f32, Box<dyn Widget>)>) -> Box<Self> {
        self.children.extend(children.into_iter().map(|(a, b)| (b, a)));
        self
    }
}

impl Widget for Row {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let mut min_size: Vector2<i32> = Vector2::zero();
        for &(ref child, _flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            min_size.y = min_size.y.max(child_min_size.y);
            min_size.x += child_min_size.x;
        }
        min_size
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|(child, _)| &**child as &dyn Widget).collect()
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FxHashMap<WidgetId, Rect<i32>>,
    ) {
        let total_flex = self.children.iter().map(|&(ref _child, flex)| flex).sum();
        let min_size = min_sizes[&self.id()];
        let own_rect = if total_flex == 0.0 {
            Rect::new(rect.start, rect.start + vec2(min_size.x, rect.size().y))
        } else {
            Rect::new(rect.start, rect.start + vec2(rect.size().x, rect.size().y))
        };
        widget_rects.insert(self.id(), own_rect);
        let mut next_pos = rect.start;
        let total_flex = if total_flex == 0.0 { 1.0 } else { total_flex };
        let extra_space = rect.size().x - min_size.x;
        for &(ref child, flex) in &self.children {
            let child_min_size = min_sizes[&child.id()];
            let widget_extra_space = (extra_space as f32 * flex / total_flex) as i32;
            let widget_width = child_min_size.x + widget_extra_space;
            let widget_rect = Rect::new(next_pos, next_pos + vec2(widget_width, rect.size().y));
            next_pos.x += widget_width;
            child.compute_rects(widget_rect, theme, min_sizes, widget_rects);
        }
    }
}

#[derive(Clone)]
pub struct TextBox {
    text: String,
    lines: Vec<String>,
    text_color: Color4,
    id: WidgetId,
}

impl TextBox {
    pub fn new(text: &str) -> Box<Self> {
        let mut res = Box::new(TextBox {
            text: text.to_owned(),
            lines: vec![],
            text_color: Color4::BLACK,
            id: WidgetId::new(),
        });
        res.update_lines();
        res
    }

    pub fn text_color(mut self: Box<Self>, color: Color4) -> Box<Self> {
        self.text_color = color;
        self
    }

    fn update_lines(&mut self) {
        self.lines = self.text.split('\n').map(|x| x.to_owned()).collect();
    }
}

impl Widget for TextBox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
        let advance_y = theme.font.advance_y();
        for (i, line) in self.lines.iter().enumerate() {
            theme.font.draw_string(
                context,
                &line,
                rect.start.cast().unwrap() + vec2(0, advance_y * i as i32),
                self.text_color,
            );
        }
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let max_width = self.lines.iter().map(|x| theme.font.string_width(context, x) as i32).max();
        if let Some(max_width) = max_width {
            vec2(max_width as i32, theme.font.advance_y() as i32 * self.lines.len() as i32)
        } else {
            vec2(0, 0)
        }
    }
}

// This is intended to be persistent, which is tricky since widgets have to own their child widgets,
// but it can be cloned.
#[derive(Clone)]
pub struct MessageBox {
    lines: Vec<(String, Color4)>,
    max_lines: usize,
    id: WidgetId,
}

impl MessageBox {
    pub fn new(max_lines: usize) -> Box<Self> {
        Box::new(MessageBox { lines: vec![], max_lines, id: WidgetId::new() })
    }

    pub fn add_line(&mut self, color: Color4, line: String) {
        self.lines.push((line, color));
        if self.lines.len() > self.max_lines {
            self.lines.remove(0);
        }
    }
}

impl Widget for MessageBox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
        let advance_y = theme.font.advance_y();
        for (i, &(ref line, color)) in self.lines.iter().enumerate() {
            theme.font.draw_string(
                context,
                &line,
                rect.start.cast().unwrap() + vec2(0, advance_y * i as i32),
                color,
            );
        }
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let max_width =
            self.lines.iter().map(|x| theme.font.string_width(context, &x.0) as i32).max();
        if let Some(max_width) = max_width {
            vec2(max_width as i32, theme.font.advance_y() as i32 * self.lines.len() as i32)
        } else {
            vec2(0, 0)
        }
    }
}

/// Allows overlapping several widgets on top of one another.
pub struct Overlap {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
}

impl Overlap {
    pub fn new() -> Box<Self> {
        Box::new(Overlap { id: WidgetId::new(), children: vec![] })
    }

    pub fn child(mut self: Box<Self>, child: Box<dyn Widget>) -> Box<Self> {
        self.children.push(child);
        self
    }

    pub fn children(mut self: Box<Self>, children: Vec<Box<dyn Widget>>) -> Box<Self> {
        self.children.extend(children.into_iter());
        self
    }
}

impl Widget for Overlap {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let mut min_size: Vector2<i32> = Vector2::zero();
        for child in &self.children {
            let child_min_size = min_sizes[&child.id()];
            min_size.x = min_size.x.max(child_min_size.x);
            min_size.y = min_size.y.max(child_min_size.y);
        }
        min_size
    }

    fn children(&self) -> Vec<&dyn Widget> {
        self.children.iter().map(|child| &**child as &dyn Widget).collect()
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FxHashMap<WidgetId, Rect<i32>>,
    ) {
        let own_rect = rect;
        widget_rects.insert(self.id(), own_rect);
        for child in &self.children {
            child.compute_rects(own_rect, theme, min_sizes, widget_rects);
        }
    }
}

#[derive(Clone)]
pub struct EmptyWidget {
    id: WidgetId,
    size: Vector2<i32>,
}

impl EmptyWidget {
    pub fn new() -> Box<Self> {
        Self::with_size(Vector2::zero())
    }

    pub fn with_size(size: Vector2<i32>) -> Box<Self> {
        Box::new(EmptyWidget { id: WidgetId::new(), size })
    }
}

impl Widget for EmptyWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        self.size
    }
}

pub struct Padding {
    id: WidgetId,
}

impl Padding {
    pub fn new() -> Box<Self> {
        Box::new(Padding { id: WidgetId::new() })
    }
}

impl Widget for Padding {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        vec2(theme.padding, theme.padding)
    }
}

pub struct Inset {
    id: WidgetId,
    child: Box<dyn Widget>,
}

impl Inset {
    pub fn new(child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Inset { id: WidgetId::new(), child })
    }
}

impl Widget for Inset {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        _rect: Rect<i32>,
        _theme: &Theme,
        _draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
    }

    fn min_size(
        &self,
        _context: &GlContext,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        min_sizes[&self.child.id()] + vec2(theme.padding * 2, theme.padding * 2)
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&*self.child]
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FxHashMap<WidgetId, Rect<i32>>,
    ) {
        widget_rects.insert(
            self.id(),
            Rect::new(rect.start, rect.end + vec2(theme.padding * 2, theme.padding * 2)),
        );
        self.child.compute_rects(
            Rect::new(
                rect.start + vec2(theme.padding, theme.padding),
                rect.end - vec2(theme.padding, theme.padding),
            ),
            theme,
            min_sizes,
            widget_rects,
        );
    }
}

/// Lets the user select one of several options, which are all shown at once.
#[derive(Clone)]
pub struct Selector<T: Copy + PartialEq> {
    options: Vec<(String, T)>,
    selected_option: Option<usize>,
    id: WidgetId,
}

impl<T: Copy + PartialEq> Selector<T> {
    pub fn new(options: Vec<(String, T)>, selected_option: Option<usize>) -> Box<Self> {
        if let Some(selected_option) = selected_option {
            assert!(selected_option < options.len());
        }
        Box::new(Self { selected_option, options, id: WidgetId::new() })
    }

    pub fn selected_option(&self) -> Option<T> {
        self.selected_option.map(|selected_option| self.options[selected_option].1)
    }

    pub fn add_option(&mut self, option: (String, T)) {
        self.options.push(option);
    }

    pub fn remove_option(&mut self, i: usize) {
        self.options.remove(i);
        if let Some(selected_option) = &mut self.selected_option {
            if *selected_option == i {
                self.selected_option = None;
            } else if *selected_option > i {
                *selected_option -= 1;
            }
        }
    }
}

impl<T: Copy + PartialEq> Widget for Selector<T> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn is_component(&self) -> bool {
        true
    }

    fn draw(
        &self,
        context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        theme: &Theme,
        draw_2d: &mut Draw2d,
        cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
        for (i, (line, _)) in self.options.iter().enumerate() {
            let pos = rect.start.cast().unwrap() + vec2(0, theme.font.advance_y() * i as i32);
            let rect = Rect::new(pos, pos + theme.font.string_size(context, &line));
            let background_color = if Some(i) == self.selected_option {
                Color4::WHITE.mul_srgb(0.5)
            } else if cursor_pos.is_some()
                && rect.contains_point(cursor_pos.unwrap().cast().unwrap())
            {
                Color4::WHITE.mul_srgb(0.75)
            } else {
                Color4::WHITE
            };
            draw_2d.fill_rect(rect, background_color);
            theme.font.draw_string(context, &line, pos, Color4::BLACK);
        }
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let max_width =
            self.options.iter().map(|(x, _)| theme.font.string_width(context, x) as i32).max();
        if let Some(max_width) = max_width {
            vec2(max_width as i32, theme.font.advance_y() as i32 * self.options.len() as i32)
        } else {
            vec2(0, 0)
        }
    }
}

pub struct SelectorResult<T: Copy + PartialEq> {
    pub selected: Option<(String, T)>,
    pub just_selected: bool,
}

impl<T: Copy + PartialEq> Component for Selector<T> {
    type Res = SelectorResult<T>;

    fn update(&mut self, theme: &Theme, events: Vec<Event>) -> Self::Res {
        let mut just_selected = false;
        for event in events {
            if let Event::MouseDown(MouseButton::Left, pos) = event {
                let entry = pos.y / theme.font.advance_y() as i32;
                assert!(
                    entry >= 0 && (entry as usize) < self.options.len(),
                    "entry {} out of range (max={})",
                    entry,
                    self.options.len()
                );
                self.selected_option = Some(entry as usize);
                just_selected = true;
            }
        }

        SelectorResult {
            selected: self
                .selected_option
                .map(|selected_option| self.options[selected_option].clone()),
            just_selected,
        }
    }
}

/// A widget that's filled with a background color.
pub struct Fill {
    id: WidgetId,
    child: Box<dyn Widget>,
    fill_color: Color4,
}

impl Fill {
    pub fn new(fill_color: Color4, child: Box<dyn Widget>) -> Box<Self> {
        Box::new(Fill { id: WidgetId::new(), child, fill_color })
    }
}

impl Widget for Fill {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw(
        &self,
        _context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        _theme: &Theme,
        draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        _is_active: bool,
    ) {
        draw_2d.fill_rect(Rect::new(rect.start, rect.end), self.fill_color);
    }

    fn min_size(
        &self,
        _context: &GlContext,
        _theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        min_sizes[&self.child.id()]
    }

    fn children(&self) -> Vec<&dyn Widget> {
        vec![&*self.child]
    }

    fn compute_rects(
        &self,
        rect: Rect<i32>,
        theme: &Theme,
        min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        widget_rects: &mut FxHashMap<WidgetId, Rect<i32>>,
    ) {
        widget_rects.insert(self.id(), Rect::new(rect.start, rect.end));
        self.child.compute_rects(Rect::new(rect.start, rect.end), theme, min_sizes, widget_rects);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TextEntryEvent {
    AddChar(char),
    Backspace,
    CaretLeft,
    CaretRight,
    EnterPressed,
}

pub struct TextEntryResult {
    pub text: Option<String>,
}

impl TextEntryResult {
    pub fn text(&self) -> Option<&str> {
        // See https://stackoverflow.com/questions/31233938/converting-from-optionstring-to-optionstr
        self.text.as_ref().map(|x| x.as_ref())
    }
}

const CARET_BLINK_RATE: f64 = 1.0;

#[derive(Clone)]
pub struct TextEntry {
    id: WidgetId,
    pub text: String,
    placeholder_text: String,
    text_color: Color4,
    caret_pos: i32,
    // TODO: support specifying the max length in pixels
    max_len: usize,
    stopwatch: Stopwatch,
    use_placeholder_text_if_empty: bool,
    continuous_updates: bool,
}

impl TextEntry {
    /// Creates a new `TextEntry`.
    ///
    /// If `continuous_updates` is enabled, the widget sends an update each time the text is
    /// changed, and isn't cleared when enter is pressed.
    pub fn new(
        start_text: &str,
        placeholder_text: &str,
        use_placeholder_text_if_empty: bool,
        max_len: usize,
        continuous_updates: bool,
    ) -> Box<Self> {
        assert!(placeholder_text.len() <= max_len);
        Box::new(TextEntry {
            id: WidgetId::new(),
            text: start_text.to_string(),
            placeholder_text: placeholder_text.to_string(),
            text_color: Color4::BLACK,
            caret_pos: 0,
            max_len,
            stopwatch: Stopwatch::new(),
            use_placeholder_text_if_empty,
            continuous_updates,
        })
    }

    pub fn text_color(mut self: Box<Self>, color: Color4) -> Box<Self> {
        self.text_color = color;
        self
    }

    pub fn cur_text(&self) -> &str {
        if self.text.is_empty() && self.use_placeholder_text_if_empty {
            &self.placeholder_text
        } else {
            &self.text
        }
    }

    /// Returns the current contents of the TextEntry, and clears the contents unless
    /// `continuous_updates` is enabled.
    fn take_cur_text(&mut self) -> String {
        if self.text.is_empty() && self.use_placeholder_text_if_empty {
            self.placeholder_text.clone()
        } else if self.continuous_updates {
            self.text.clone()
        } else {
            mem::take(&mut self.text)
        }
    }
}

impl Component for TextEntry {
    type Res = TextEntryResult;

    fn update(&mut self, _theme: &Theme, events: Vec<Event>) -> TextEntryResult {
        let mut res = None;
        for event in events {
            match event {
                Event::KeyDown(key) => match key.code.as_ref() {
                    "Backspace" => {
                        if self.caret_pos > 0 {
                            self.text.remove(self.caret_pos as usize - 1);
                            self.caret_pos -= 1;
                        }
                    }
                    "ArrowLeft" => self.caret_pos = (self.caret_pos - 1).max(0),
                    "ArrowRight" => {
                        self.caret_pos = (self.caret_pos + 1).min(self.text.len() as i32)
                    }
                    "Enter" => {
                        res = Some(self.take_cur_text());
                        self.caret_pos = 0;
                    }
                    _ => (),
                },
                Event::CharEntered(c) => {
                    if self.text.len() < self.max_len {
                        self.text.insert(self.caret_pos as usize, c);
                        self.caret_pos += 1;
                    }
                }
                _ => (),
            }
        }
        if self.continuous_updates {
            res = Some(self.cur_text().to_owned());
        }
        TextEntryResult { text: res }
    }
}

impl Widget for TextEntry {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn is_component(&self) -> bool {
        true
    }

    fn draw(
        &self,
        context: &GlContext,
        _surface: &dyn Surface,
        rect: Rect<i32>,
        theme: &Theme,
        draw_2d: &mut Draw2d,
        _cursor_pos: Option<Point2<i32>>,
        is_active: bool,
    ) {
        let fill_color = theme.button_fill_color;
        let (drawn_text, drawn_text_color) = if self.text.is_empty() {
            (&self.placeholder_text, theme.button_text_color * 0.8)
        } else {
            (&self.text, theme.button_text_color)
        };
        draw_2d.fill_rect(rect, fill_color);
        draw_2d.outline_rect(rect, theme.button_border_color, 1.0);
        theme.font.draw_string(context, &drawn_text, rect.start + vec2(2, 1), drawn_text_color);
        if self.stopwatch.get_time().rem_euclid(CARET_BLINK_RATE) < CARET_BLINK_RATE * 0.5
            && is_active
        {
            let caret_x_offset =
                theme.font.string_width(context, &drawn_text[0..self.caret_pos as usize]) + 2.0;
            draw_2d.draw_line(
                point2(caret_x_offset + rect.start.x as f32, rect.start.y as f32 + 2.0),
                point2(caret_x_offset + rect.start.x as f32, rect.end.y as f32 - 2.0),
                theme.button_text_color,
                1.0,
            );
        }
    }

    fn min_size(
        &self,
        context: &GlContext,
        theme: &Theme,
        _min_sizes: &FxHashMap<WidgetId, Vector2<i32>>,
        _window_size: Vector2<i32>,
    ) -> Vector2<i32> {
        let drawn_text = if self.text.is_empty() { &self.placeholder_text } else { &self.text };
        theme.font.string_size(context, drawn_text) + vec2(4, 2)
    }
}
