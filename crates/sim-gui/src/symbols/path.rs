//! Minimal SVG path parser and curve flattener.

use super::svg::{Point, SvgError};

const CURVE_STEPS: usize = 16;

/// One open or closed SVG subpath.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Subpath {
    pub points: Vec<Point>,
    pub closed: bool,
}

/// Parses an SVG path data string into subpaths.
pub(crate) fn parse(data: &str) -> Result<Vec<Subpath>, SvgError> {
    let mut parser = PathParser::new(data);
    let mut state = PathState::new();
    while !parser.done() {
        parser.skip_separators();
        if parser.done() {
            break;
        }
        state.read_command(&mut parser)?;
        state.step(&mut parser)?;
    }
    state.finish()
}

struct PathState {
    subpaths: Vec<Subpath>,
    current: Subpath,
    start: Point,
    cursor: Point,
    command: Option<u8>,
    last_cubic: Option<Point>,
    last_quad: Option<Point>,
}

impl PathState {
    fn new() -> Self {
        Self {
            subpaths: Vec::new(),
            current: Subpath::default(),
            start: Point::ZERO,
            cursor: Point::ZERO,
            command: None,
            last_cubic: None,
            last_quad: None,
        }
    }

    fn read_command(&mut self, parser: &mut PathParser<'_>) -> Result<(), SvgError> {
        if parser.at_command() {
            self.command = Some(parser.read_command()?);
        }
        Ok(())
    }

    fn step(&mut self, parser: &mut PathParser<'_>) -> Result<(), SvgError> {
        let command = self.command.ok_or(SvgError::MissingCommand)?;
        match command {
            b'M' | b'm' => self.move_command(parser, command == b'm'),
            b'L' | b'l' => self.line_command(parser, command == b'l'),
            b'H' | b'h' => self.horizontal_command(parser, command == b'h'),
            b'V' | b'v' => self.vertical_command(parser, command == b'v'),
            b'C' | b'c' => self.cubic_command(parser, command == b'c'),
            b'S' | b's' => self.smooth_cubic_command(parser, command == b's'),
            b'Q' | b'q' => self.quad_command(parser, command == b'q'),
            b'T' | b't' => self.smooth_quad_command(parser, command == b't'),
            b'A' | b'a' => Err(SvgError::UnsupportedCommand('A')),
            b'Z' | b'z' => self.close_command(),
            _ => Err(SvgError::UnknownCommand(command as char)),
        }
    }

    fn move_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let point = self.read_point(parser, relative)?;
        self.move_to(point);
        self.command = Some(if relative { b'l' } else { b'L' });
        self.clear_curves();
        Ok(())
    }

    fn line_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let point = self.read_point(parser, relative)?;
        self.line_to(point);
        self.clear_curves();
        Ok(())
    }

    fn horizontal_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let x = parser.read_number()?;
        let x = if relative { self.cursor.x + x } else { x };
        self.line_to(Point::new(x, self.cursor.y));
        self.clear_curves();
        Ok(())
    }

    fn vertical_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let y = parser.read_number()?;
        let y = if relative { self.cursor.y + y } else { y };
        self.line_to(Point::new(self.cursor.x, y));
        self.clear_curves();
        Ok(())
    }

    fn cubic_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let control_a = self.read_point(parser, relative)?;
        let control_b = self.read_point(parser, relative)?;
        let end = self.read_point(parser, relative)?;
        self.cubic_to(control_a, control_b, end);
        self.last_cubic = Some(control_b);
        self.last_quad = None;
        Ok(())
    }

    fn smooth_cubic_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let control_a = self
            .last_cubic
            .map_or(self.cursor, |point| reflect(point, self.cursor));
        let control_b = self.read_point(parser, relative)?;
        let end = self.read_point(parser, relative)?;
        self.cubic_to(control_a, control_b, end);
        self.last_cubic = Some(control_b);
        self.last_quad = None;
        Ok(())
    }

    fn quad_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let control = self.read_point(parser, relative)?;
        let end = self.read_point(parser, relative)?;
        self.quad_to(control, end);
        self.last_quad = Some(control);
        self.last_cubic = None;
        Ok(())
    }

    fn smooth_quad_command(
        &mut self,
        parser: &mut PathParser<'_>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let control = self
            .last_quad
            .map_or(self.cursor, |point| reflect(point, self.cursor));
        let end = self.read_point(parser, relative)?;
        self.quad_to(control, end);
        self.last_quad = Some(control);
        self.last_cubic = None;
        Ok(())
    }

    fn close_command(&mut self) -> Result<(), SvgError> {
        self.close_current();
        self.command = None;
        self.clear_curves();
        self.cursor = self.start;
        Ok(())
    }

    fn read_point(&self, parser: &mut PathParser<'_>, relative: bool) -> Result<Point, SvgError> {
        let point = parser.read_pair()?;
        Ok(if relative {
            self.cursor.add(point)
        } else {
            point
        })
    }

    fn move_to(&mut self, point: Point) {
        self.finish_current();
        self.start = point;
        self.cursor = point;
        self.current = Subpath {
            points: vec![point],
            closed: false,
        };
    }

    fn line_to(&mut self, point: Point) {
        if self.current.points.is_empty() {
            self.current.points.push(self.cursor);
        }
        self.current.points.push(point);
        self.cursor = point;
    }

    fn cubic_to(&mut self, control_a: Point, control_b: Point, end: Point) {
        if self.current.points.is_empty() {
            self.current.points.push(self.cursor);
        }
        let points = cubic_points(self.cursor, control_a, control_b, end);
        self.current.points.extend(points);
        self.cursor = end;
    }

    fn quad_to(&mut self, control: Point, end: Point) {
        if self.current.points.is_empty() {
            self.current.points.push(self.cursor);
        }
        let points = quad_points(self.cursor, control, end);
        self.current.points.extend(points);
        self.cursor = end;
    }

    fn close_current(&mut self) {
        if self.current.points.is_empty() {
            return;
        }
        self.current.closed = true;
        self.finish_current();
        self.current = Subpath {
            points: vec![self.start],
            closed: false,
        };
    }

    fn finish_current(&mut self) {
        if self.current.points.is_empty() {
            return;
        }
        let current = std::mem::take(&mut self.current);
        self.subpaths.push(current);
    }

    fn finish(mut self) -> Result<Vec<Subpath>, SvgError> {
        self.finish_current();
        Ok(self.subpaths)
    }

    fn clear_curves(&mut self) {
        self.last_cubic = None;
        self.last_quad = None;
    }
}

struct PathParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PathParser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            data: data.as_bytes(),
            pos: 0,
        }
    }

    fn done(&self) -> bool {
        self.pos >= self.data.len()
    }

    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn skip_separators(&mut self) {
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_whitespace() || byte == b',')
        {
            self.pos += 1;
        }
    }

    fn at_command(&self) -> bool {
        self.peek().is_some_and(is_command)
    }

    fn read_command(&mut self) -> Result<u8, SvgError> {
        let byte = self.peek().ok_or(SvgError::MissingCommand)?;
        if !is_command(byte) {
            return Err(SvgError::MissingCommand);
        }
        self.pos += 1;
        Ok(byte)
    }

    fn read_pair(&mut self) -> Result<Point, SvgError> {
        let x = self.read_number()?;
        let y = self.read_number()?;
        Ok(Point::new(x, y))
    }

    fn read_number(&mut self) -> Result<f32, SvgError> {
        self.skip_separators();
        let start = self.pos;
        self.consume_sign();
        self.consume_digits();
        self.consume_decimal();
        self.consume_digits();
        self.consume_exponent();
        if self.pos == start {
            return Err(SvgError::MissingNumber);
        }
        self.parse_number(start)
    }

    fn parse_number(&self, start: usize) -> Result<f32, SvgError> {
        let text = std::str::from_utf8(&self.data[start..self.pos])
            .map_err(|_| SvgError::InvalidNumber)?;
        let value = text.parse::<f32>().map_err(|_| SvgError::InvalidNumber)?;
        if !value.is_finite() {
            return Err(SvgError::InvalidNumber);
        }
        Ok(value)
    }

    fn consume_sign(&mut self) {
        if self.peek().is_some_and(|byte| byte == b'+' || byte == b'-') {
            self.pos += 1;
        }
    }

    fn consume_digits(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.pos += 1;
        }
    }

    fn consume_decimal(&mut self) {
        if self.peek() == Some(b'.') {
            self.pos += 1;
        }
    }

    fn consume_exponent(&mut self) {
        if !self.peek().is_some_and(|byte| byte == b'e' || byte == b'E') {
            return;
        }
        self.pos += 1;
        self.consume_sign();
        self.consume_digits();
    }
}

fn is_command(byte: u8) -> bool {
    b"MmLlHhVvCcSsQqTtAaZz".contains(&byte)
}

fn reflect(control: Point, cursor: Point) -> Point {
    Point::new(2.0 * cursor.x - control.x, 2.0 * cursor.y - control.y)
}

fn cubic_points(start: Point, a: Point, b: Point, end: Point) -> impl Iterator<Item = Point> {
    (1..=CURVE_STEPS).map(move |index| {
        let t = index as f32 / CURVE_STEPS as f32;
        cubic_point(start, a, b, end, t)
    })
}

fn cubic_point(start: Point, a: Point, b: Point, end: Point, t: f32) -> Point {
    let u = 1.0 - t;
    let weights = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
    Point::new(
        start.x * weights[0] + a.x * weights[1] + b.x * weights[2] + end.x * weights[3],
        start.y * weights[0] + a.y * weights[1] + b.y * weights[2] + end.y * weights[3],
    )
}

fn quad_points(start: Point, control: Point, end: Point) -> impl Iterator<Item = Point> {
    (1..=CURVE_STEPS).map(move |index| {
        let t = index as f32 / CURVE_STEPS as f32;
        quad_point(start, control, end, t)
    })
}

fn quad_point(start: Point, control: Point, end: Point, t: f32) -> Point {
    let u = 1.0 - t;
    Point::new(
        u * u * start.x + 2.0 * u * t * control.x + t * t * end.x,
        u * u * start.y + 2.0 * u * t * control.y + t * t * end.y,
    )
}
