//! Small SVG subset parser for embedded schematic symbols.

use super::path::{self, Subpath};

const CIRCLE_SEGMENTS: usize = 64;

/// One point in SVG user units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub(crate) const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(crate) const fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn transformed(self, transform: Transform) -> Self {
        transform.apply(self)
    }
}

/// Bounding box of a parsed symbol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Bounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl Bounds {
    const EMPTY: Self = Self {
        min_x: f32::INFINITY,
        min_y: f32::INFINITY,
        max_x: f32::NEG_INFINITY,
        max_y: f32::NEG_INFINITY,
    };

    fn include(&mut self, point: Point) {
        self.min_x = self.min_x.min(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_x = self.max_x.max(point.x);
        self.max_y = self.max_y.max(point.y);
    }

    fn is_valid(self) -> bool {
        self.width() > 0.0 && self.height() > 0.0 && self.width().is_finite()
    }

    pub(crate) fn width(self) -> f32 {
        self.max_x - self.min_x
    }

    pub(crate) fn height(self) -> f32 {
        self.max_y - self.min_y
    }

    pub(crate) fn center(self) -> Point {
        Point::new(
            (self.min_x + self.max_x) * 0.5,
            (self.min_y + self.max_y) * 0.5,
        )
    }
}

/// One painted SVG subpath in user units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SvgShape {
    pub points: Vec<Point>,
    pub closed: bool,
    pub fill: Option<f32>,
    pub stroke: Option<f32>,
    pub width: f32,
}

/// A parsed symbol with its drawn bounds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SvgSymbol {
    pub shapes: Vec<SvgShape>,
    pub bounds: Bounds,
}

/// Parse failures for the supported SVG subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SvgError {
    MissingTagEnd,
    MissingAttribute,
    InvalidNumber,
    MissingNumber,
    MissingCommand,
    UnknownCommand(char),
    Empty,
    InvalidBounds,
}

/// Parses an SVG document into painted subpaths.
pub(crate) fn parse(text: &str) -> Result<SvgSymbol, SvgError> {
    Parser::new(text).parse()
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
    shapes: Vec<SvgShape>,
    bounds: Bounds,
    stack: Vec<Frame>,
    ignore_depth: usize,
}

#[derive(Clone, Copy)]
struct Frame {
    style: Style,
    transform: Transform,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            shapes: Vec::new(),
            bounds: Bounds::EMPTY,
            stack: Vec::new(),
            ignore_depth: 0,
        }
    }

    fn parse(mut self) -> Result<SvgSymbol, SvgError> {
        while let Some(tag) = self.next_tag()? {
            if tag.closing {
                self.close_tag(&tag);
            } else {
                self.open_tag(&tag)?;
            }
        }
        if self.shapes.is_empty() {
            return Err(SvgError::Empty);
        }
        if !self.bounds.is_valid() {
            return Err(SvgError::InvalidBounds);
        }
        Ok(SvgSymbol {
            shapes: self.shapes,
            bounds: self.bounds,
        })
    }

    fn next_tag(&mut self) -> Result<Option<Tag<'a>>, SvgError> {
        loop {
            let Some(offset) = self.input[self.pos..].find('<') else {
                return Ok(None);
            };
            self.pos += offset;
            if self.skip_special()? {
                continue;
            }
            let end = self.tag_end()?;
            let content = &self.input[self.pos + 1..end];
            self.pos = end + 1;
            return Tag::parse(content).map(Some);
        }
    }

    fn skip_special(&mut self) -> Result<bool, SvgError> {
        if self.input[self.pos..].starts_with("<!--") {
            self.skip_until("-->")?;
            return Ok(true);
        }
        if self.input[self.pos..].starts_with("<?") {
            self.skip_until("?>")?;
            return Ok(true);
        }
        if self.input[self.pos..].starts_with("<!") {
            self.skip_until(">")?;
            return Ok(true);
        }
        Ok(false)
    }

    fn skip_until(&mut self, needle: &str) -> Result<(), SvgError> {
        let offset = self.input[self.pos..]
            .find(needle)
            .ok_or(SvgError::MissingTagEnd)?;
        self.pos += offset + needle.len();
        Ok(())
    }

    fn tag_end(&self) -> Result<usize, SvgError> {
        let mut quote = None;
        for (offset, byte) in self.input[self.pos + 1..].bytes().enumerate() {
            match (quote, byte) {
                (Some(expected), value) if value == expected => quote = None,
                (None, b'"' | b'\'') => quote = Some(byte),
                (None, b'>') => return Ok(self.pos + 1 + offset),
                _ => {}
            }
        }
        Err(SvgError::MissingTagEnd)
    }

    fn open_tag(&mut self, tag: &Tag<'a>) -> Result<(), SvgError> {
        if self.ignore_depth > 0 {
            self.adjust_ignore(tag);
            return Ok(());
        }
        let name = local_name(tag.name);
        if is_ignored(name) {
            self.adjust_ignore(tag);
            return Ok(());
        }
        let style = self.current_style().with_attrs(tag);
        let transform = self.current_transform().with_attrs(tag);
        if name == "g" || name == "svg" {
            self.stack.push(Frame { style, transform });
            return Ok(());
        }
        self.push_shape(name, tag, style, transform)
    }

    fn adjust_ignore(&mut self, tag: &Tag<'a>) {
        if !tag.self_closing {
            self.ignore_depth += 1;
        }
    }

    fn close_tag(&mut self, tag: &Tag<'a>) {
        if self.ignore_depth > 0 {
            self.ignore_depth -= 1;
            return;
        }
        let name = local_name(tag.name);
        if name == "g" || name == "svg" {
            self.stack.pop();
        }
    }

    fn current_style(&self) -> Style {
        self.stack
            .last()
            .map_or_else(Style::default, |frame| frame.style)
    }

    fn current_transform(&self) -> Transform {
        self.stack
            .last()
            .map_or(Transform::IDENTITY, |frame| frame.transform)
    }

    fn push_shape(
        &mut self,
        name: &str,
        tag: &Tag<'a>,
        style: Style,
        transform: Transform,
    ) -> Result<(), SvgError> {
        match name {
            "path" => return self.push_path(tag, style, transform),
            "line" => self.push_line(tag, style, transform),
            "rect" => self.push_rect(tag, style, transform),
            "circle" => self.push_circle(tag, style, transform),
            "ellipse" => self.push_ellipse(tag, style, transform),
            "polyline" => self.push_poly(tag, style, transform, false),
            "polygon" => self.push_poly(tag, style, transform, true),
            _ => {}
        }
        Ok(())
    }

    fn push_path(
        &mut self,
        tag: &Tag<'a>,
        style: Style,
        transform: Transform,
    ) -> Result<(), SvgError> {
        let Some(data) = tag.attr("d") else {
            return Ok(());
        };
        for subpath in path::parse(data)? {
            self.push_subpath(subpath, style, transform);
        }
        Ok(())
    }

    fn push_subpath(&mut self, subpath: Subpath, style: Style, transform: Transform) {
        if subpath.points.len() < 2 {
            return;
        }
        self.push_points(subpath.points, subpath.closed, style, transform);
    }

    fn push_points(
        &mut self,
        points: Vec<Point>,
        closed: bool,
        style: Style,
        transform: Transform,
    ) {
        let (fill, stroke) = (style.fill_alpha(), style.stroke_alpha());
        if fill.is_none() && stroke.is_none() {
            return;
        }
        let points = self.transform_points(points, transform);
        self.shapes.push(SvgShape {
            points,
            closed,
            fill,
            stroke,
            width: style.width * transform.scale_factor(),
        });
    }

    fn transform_points(&mut self, points: Vec<Point>, transform: Transform) -> Vec<Point> {
        let points: Vec<Point> = points
            .into_iter()
            .map(|point| point.transformed(transform))
            .collect();
        for point in &points {
            self.bounds.include(*point);
        }
        points
    }

    fn push_line(&mut self, tag: &Tag<'a>, style: Style, transform: Transform) {
        let start = Point::new(
            tag.length("x1").unwrap_or(0.0),
            tag.length("y1").unwrap_or(0.0),
        );
        let end = Point::new(
            tag.length("x2").unwrap_or(0.0),
            tag.length("y2").unwrap_or(0.0),
        );
        self.push_points(vec![start, end], false, style, transform);
    }

    fn push_rect(&mut self, tag: &Tag<'a>, style: Style, transform: Transform) {
        let x = tag.length("x").unwrap_or(0.0);
        let y = tag.length("y").unwrap_or(0.0);
        let width = tag.length("width").unwrap_or(0.0);
        let height = tag.length("height").unwrap_or(0.0);
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let points = vec![
            Point::new(x, y),
            Point::new(x + width, y),
            Point::new(x + width, y + height),
            Point::new(x, y + height),
        ];
        self.push_points(points, true, style, transform);
    }

    fn push_circle(&mut self, tag: &Tag<'a>, style: Style, transform: Transform) {
        let center = Point::new(
            tag.length("cx").unwrap_or(0.0),
            tag.length("cy").unwrap_or(0.0),
        );
        let radius = tag.length("r").unwrap_or(0.0);
        if radius <= 0.0 {
            return;
        }
        let points = ellipse_points(center, radius, radius);
        self.push_points(points, true, style, transform);
    }

    fn push_ellipse(&mut self, tag: &Tag<'a>, style: Style, transform: Transform) {
        let center = Point::new(
            tag.length("cx").unwrap_or(0.0),
            tag.length("cy").unwrap_or(0.0),
        );
        let rx = tag.length("rx").unwrap_or(0.0);
        let ry = tag.length("ry").unwrap_or(0.0);
        if rx <= 0.0 || ry <= 0.0 {
            return;
        }
        let points = ellipse_points(center, rx, ry);
        self.push_points(points, true, style, transform);
    }

    fn push_poly(&mut self, tag: &Tag<'a>, style: Style, transform: Transform, closed: bool) {
        let Some(text) = tag.attr("points") else {
            return;
        };
        let points = parse_points(text);
        if points.len() < 2 {
            return;
        }
        self.push_points(points, closed, style, transform);
    }
}

struct Tag<'a> {
    name: &'a str,
    closing: bool,
    self_closing: bool,
    attrs: Vec<(&'a str, &'a str)>,
}

impl<'a> Tag<'a> {
    fn parse(content: &'a str) -> Result<Self, SvgError> {
        let (text, closing, self_closing) = split_tag(content);
        let end = text.find(char::is_whitespace).unwrap_or(text.len());
        let name = &text[..end];
        if name.is_empty() {
            return Err(SvgError::MissingAttribute);
        }
        Ok(Self {
            name,
            closing,
            self_closing,
            attrs: parse_attrs(&text[end..]),
        })
    }

    fn attr(&self, name: &str) -> Option<&'a str> {
        self.attrs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    }

    fn length(&self, name: &str) -> Option<f32> {
        self.attr(name).and_then(parse_length)
    }
}

fn split_tag(content: &str) -> (&str, bool, bool) {
    let mut text = content.trim();
    let closing = text.starts_with('/');
    if closing {
        text = text[1..].trim_start();
    }
    let self_closing = text.ends_with('/');
    if self_closing {
        text = text[..text.len() - 1].trim_end();
    }
    (text, closing, self_closing)
}

struct AttrParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> AttrParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn next_attr(&mut self) -> Option<(&'a str, &'a str)> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }
        let name = self.read_name();
        self.skip_whitespace();
        if self.peek() != Some(b'=') {
            return Some((name, ""));
        }
        self.pos += 1;
        self.skip_whitespace();
        Some((name, self.read_value()))
    }

    fn read_name(&mut self) -> &'a str {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|byte| !byte.is_ascii_whitespace() && byte != b'=')
        {
            self.pos += 1;
        }
        &self.input[start..self.pos]
    }

    fn read_value(&mut self) -> &'a str {
        let Some(quote) = self.peek().filter(|byte| *byte == b'"' || *byte == b'\'') else {
            return self.read_unquoted();
        };
        self.pos += 1;
        let start = self.pos;
        while self.peek().is_some_and(|byte| byte != quote) {
            self.pos += 1;
        }
        let value = &self.input[start..self.pos];
        if self.peek() == Some(quote) {
            self.pos += 1;
        }
        value
    }

    fn read_unquoted(&mut self) -> &'a str {
        let start = self.pos;
        while self.peek().is_some_and(|byte| !byte.is_ascii_whitespace()) {
            self.pos += 1;
        }
        &self.input[start..self.pos]
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }
}

fn parse_attrs(text: &str) -> Vec<(&str, &str)> {
    let mut parser = AttrParser::new(text);
    let mut attrs = Vec::new();
    while let Some(attr) = parser.next_attr() {
        attrs.push(attr);
    }
    attrs
}

#[derive(Clone, Copy)]
struct Style {
    fill: Paint,
    stroke: Paint,
    fill_opacity: f32,
    stroke_opacity: f32,
    opacity: f32,
    width: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Paint {
    None,
    Solid,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            fill: Paint::Solid,
            stroke: Paint::None,
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
            opacity: 1.0,
            width: 1.0,
        }
    }
}

impl Style {
    fn with_attrs(mut self, tag: &Tag<'_>) -> Self {
        for name in [
            "fill",
            "stroke",
            "stroke-width",
            "fill-opacity",
            "stroke-opacity",
            "opacity",
            "display",
            "visibility",
        ] {
            if let Some(value) = tag.attr(name) {
                self.apply(name, value);
            }
        }
        if let Some(css) = tag.attr("style") {
            self.apply_css(css);
        }
        self
    }

    fn apply_css(&mut self, css: &str) {
        for declaration in css.split(';') {
            let Some((name, value)) = declaration.split_once(':') else {
                continue;
            };
            self.apply(name.trim(), value.trim());
        }
    }

    fn apply(&mut self, name: &str, value: &str) {
        match name {
            "fill" => self.fill = paint(value),
            "stroke" => self.stroke = paint(value),
            "stroke-width" => self.width = parse_length(value).map_or(self.width, |v| v.max(0.0)),
            "fill-opacity" => self.fill_opacity = parse_opacity(value).unwrap_or(self.fill_opacity),
            "stroke-opacity" => {
                self.stroke_opacity = parse_opacity(value).unwrap_or(self.stroke_opacity);
            }
            "opacity" => self.opacity = parse_opacity(value).unwrap_or(self.opacity),
            "display" | "visibility" => self.apply_visibility(value),
            _ => {}
        }
    }

    fn apply_visibility(&mut self, value: &str) {
        if value == "none" || value == "hidden" {
            self.fill = Paint::None;
            self.stroke = Paint::None;
        }
    }

    fn fill_alpha(self) -> Option<f32> {
        paint_alpha(self.fill, self.fill_opacity * self.opacity)
    }

    fn stroke_alpha(self) -> Option<f32> {
        paint_alpha(self.stroke, self.stroke_opacity * self.opacity)
    }
}

fn paint(value: &str) -> Paint {
    if value.trim().eq_ignore_ascii_case("none") {
        Paint::None
    } else {
        Paint::Solid
    }
}

fn paint_alpha(paint: Paint, alpha: f32) -> Option<f32> {
    if paint == Paint::Solid && alpha > 0.01 {
        Some(alpha.clamp(0.0, 1.0))
    } else {
        None
    }
}

fn parse_opacity(value: &str) -> Option<f32> {
    parse_length(value).map(|value| value.clamp(0.0, 1.0))
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Transform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Transform {
    const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    fn apply(self, point: Point) -> Point {
        Point::new(
            self.a * point.x + self.c * point.y + self.e,
            self.b * point.x + self.d * point.y + self.f,
        )
    }

    fn mul(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    fn translate(tx: f32, ty: f32) -> Self {
        Self {
            e: tx,
            f: ty,
            ..Self::IDENTITY
        }
    }

    fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            d: sy,
            ..Self::IDENTITY
        }
    }

    fn rotate(angle: f32) -> Self {
        let radians = angle.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            ..Self::IDENTITY
        }
    }

    fn rotate_around(angle: f32, cx: f32, cy: f32) -> Self {
        Self::translate(cx, cy)
            .mul(Self::rotate(angle))
            .mul(Self::translate(-cx, -cy))
    }

    fn matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    fn skew_x(angle: f32) -> Self {
        Self {
            c: angle.to_radians().tan(),
            ..Self::IDENTITY
        }
    }

    fn skew_y(angle: f32) -> Self {
        Self {
            b: angle.to_radians().tan(),
            ..Self::IDENTITY
        }
    }

    fn scale_factor(self) -> f32 {
        (self.a * self.d - self.b * self.c).abs().sqrt()
    }

    fn with_attrs(mut self, tag: &Tag<'_>) -> Self {
        let Some(value) = tag.attr("transform") else {
            return self;
        };
        if let Ok(child) = parse_transform(value) {
            self = self.mul(child);
        }
        self
    }
}

fn parse_transform(text: &str) -> Result<Transform, SvgError> {
    let mut parser = TransformParser::new(text);
    let mut result = Transform::IDENTITY;
    while let Some((name, args)) = parser.next_function()? {
        result = result.mul(function_transform(name, &args));
    }
    Ok(result)
}

fn function_transform(name: &str, args: &[f32]) -> Transform {
    match name {
        "translate" => {
            Transform::translate(args.first().copied().unwrap_or(0.0), arg(args, 1, 0.0))
        }
        "scale" => {
            let sx = args.first().copied().unwrap_or(1.0);
            Transform::scale(sx, arg(args, 1, sx))
        }
        "rotate" => rotate_transform(args),
        "matrix" if args.len() >= 6 => {
            Transform::matrix(args[0], args[1], args[2], args[3], args[4], args[5])
        }
        "skewX" => Transform::skew_x(args.first().copied().unwrap_or(0.0)),
        "skewY" => Transform::skew_y(args.first().copied().unwrap_or(0.0)),
        _ => Transform::IDENTITY,
    }
}

fn rotate_transform(args: &[f32]) -> Transform {
    let angle = args.first().copied().unwrap_or(0.0);
    if args.len() >= 3 {
        return Transform::rotate_around(angle, args[1], args[2]);
    }
    Transform::rotate(angle)
}

fn arg(args: &[f32], index: usize, default: f32) -> f32 {
    args.get(index).copied().unwrap_or(default)
}

struct TransformParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> TransformParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn next_function(&mut self) -> Result<Option<(&'a str, Vec<f32>)>, SvgError> {
        self.skip_separators();
        if self.pos >= self.input.len() {
            return Ok(None);
        }
        let name = self.read_name();
        self.skip_whitespace();
        if self.peek() != Some('(') {
            return Err(SvgError::MissingAttribute);
        }
        self.pos += 1;
        Ok(Some((name, self.read_args()?)))
    }

    fn read_name(&mut self) -> &'a str {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|value| value != '(' && !value.is_whitespace())
        {
            self.pos += 1;
        }
        &self.input[start..self.pos]
    }

    fn read_args(&mut self) -> Result<Vec<f32>, SvgError> {
        let start = self.pos;
        let Some(offset) = self.input[self.pos..].find(')') else {
            return Err(SvgError::MissingTagEnd);
        };
        self.pos = start + offset + 1;
        Ok(parse_numbers(&self.input[start..start + offset]))
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
    }

    fn skip_separators(&mut self) {
        while self
            .peek()
            .is_some_and(|value| value.is_whitespace() || value == ',')
        {
            self.pos += 1;
        }
    }
}

fn parse_numbers(text: &str) -> Vec<f32> {
    text.split(|value: char| value.is_ascii_whitespace() || value == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f32>().ok())
        .collect()
}

fn parse_points(text: &str) -> Vec<Point> {
    parse_numbers(text)
        .chunks_exact(2)
        .map(|pair| Point::new(pair[0], pair[1]))
        .collect()
}

fn ellipse_points(center: Point, rx: f32, ry: f32) -> Vec<Point> {
    (0..CIRCLE_SEGMENTS)
        .map(|index| {
            let angle = index as f32 / CIRCLE_SEGMENTS as f32 * std::f32::consts::TAU;
            Point::new(center.x + rx * angle.cos(), center.y + ry * angle.sin())
        })
        .collect()
}

fn parse_length(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    let end = trimmed
        .find(|value: char| value.is_ascii_alphabetic() || value == '%')
        .unwrap_or(trimmed.len());
    trimmed[..end]
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

fn is_ignored(name: &str) -> bool {
    const IGNORED: &str = "defs metadata title desc style text tspan namedview \
        perspective clipPath mask pattern linearGradient radialGradient filter marker symbol";
    IGNORED.split_ascii_whitespace().any(|value| value == name)
}
