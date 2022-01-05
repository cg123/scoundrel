use anyhow::{anyhow, Result};
use scoundrel_geometry::{Axis2D, Point, Rect};

#[derive(Debug, Copy, Clone)]
pub enum PanelSize {
    Fit,
    Fill,
    Fixed(Point),
}
#[derive(Debug, Copy, Clone)]
pub enum AnchorEdge {
    Top,
    Left,
    Bottom,
    Right,
}
#[derive(Debug, Clone)]
pub enum LayoutKind {
    Panel { size: PanelSize, margin: i32 },
    Centered { dimension: Option<Axis2D> },
    Anchored { edge: AnchorEdge },
    Stack { dimension: Axis2D },
}

fn stack_layout<T>(stack: &LayoutElement<T>, available: Rect) -> Result<(Point, Vec<Rect>)> {
    let dimension = match stack.kind {
        LayoutKind::Stack { dimension } => dimension,
        _ => panic!("not a stack"),
    };
    let mut child_rects = vec![];
    let mut remaining = available;
    for child in &stack.children {
        let size = child.size(remaining)?;
        child_rects.push(Rect::with_size(remaining.min, size));

        let mut offset = Point::zero();
        offset[dimension] = size[dimension];
        remaining = remaining.sub_rect(offset, remaining.size());
    }

    let mut size = available.size();
    size[dimension] = remaining.min[dimension] - available.min[dimension];
    Ok((size, child_rects))
}

pub struct LayoutElement<Data = ()> {
    pub kind: LayoutKind,
    pub children: Vec<LayoutElement<Data>>,
    pub rect: Rect,
    pub data: Option<Data>,
}

pub trait LayoutMutVisitor<Data> {
    fn call(&mut self, element: &mut LayoutElement<Data>);
}
impl<Data, T> LayoutMutVisitor<Data> for T
where
    T: FnMut(&mut LayoutElement<Data>),
{
    fn call(&mut self, element: &mut LayoutElement<Data>) {
        self(element);
    }
}
pub trait LayoutVisitor<Data> {
    fn call(&mut self, data: &Data, element: &LayoutElement<Data>);
}
impl<Data, T> LayoutVisitor<Data> for T
where
    T: FnMut(&Data, &LayoutElement<Data>),
{
    fn call(&mut self, data: &Data, element: &LayoutElement<Data>) {
        self(data, element);
    }
}

impl<Data> LayoutElement<Data> {
    pub fn new(kind: LayoutKind) -> LayoutElement<Data> {
        LayoutElement {
            kind,
            children: Vec::new(),
            rect: Rect::with_points(Point::zero(), Point::zero()),
            data: None,
        }
    }
    pub fn with_child(mut self, child: LayoutElement<Data>) -> Self {
        self.children.push(child);
        self
    }
    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    pub fn inner_mut(&mut self) -> Result<&mut LayoutElement<Data>> {
        if self.children.len() == 1 {
            Ok(&mut self.children[0])
        } else {
            Err(anyhow!("expected inner element, none present"))
        }
    }
    pub fn inner(&self) -> Result<&LayoutElement<Data>> {
        if self.children.len() == 1 {
            Ok(&self.children[0])
        } else {
            Err(anyhow!("expected inner element, none present"))
        }
    }

    pub fn child_rects(&self, given_size: Rect) -> Result<Vec<Rect>> {
        let res = match self.kind {
            LayoutKind::Panel { size: _, margin } => {
                let m = Point::new(margin, margin);
                let inner_avail = given_size.sub_rect(m, given_size.size() - m * 2);
                vec![inner_avail]
            }
            LayoutKind::Centered { dimension } => {
                let inner_size = self.inner()?.size(given_size)?;
                vec![match dimension {
                    Some(dim) => {
                        let mut min_pt = given_size.min;
                        min_pt[dim] += (given_size.size()[dim] - inner_size[dim]) / 2;
                        Rect::with_points(min_pt, min_pt + inner_size)
                    }
                    None => Rect::with_center((given_size.min + given_size.max) / 2, inner_size),
                }]
            }
            LayoutKind::Anchored { edge } => {
                let inner_size = self.inner()?.size(given_size)?;
                vec![match edge {
                    AnchorEdge::Top => Rect::with_points(
                        given_size.min,
                        Point::new(given_size.max.x, given_size.min.y + inner_size.y),
                    ),
                    AnchorEdge::Bottom => Rect::with_points(
                        Point::new(given_size.min.x, given_size.max.y - inner_size.y),
                        given_size.max,
                    ),
                    AnchorEdge::Left => Rect::with_points(
                        given_size.min,
                        Point::new(given_size.min.x + inner_size.x, given_size.max.y),
                    ),
                    AnchorEdge::Right => Rect::with_points(
                        Point::new(given_size.max.x - inner_size.x, given_size.min.y),
                        given_size.max,
                    ),
                }]
            }
            LayoutKind::Stack { .. } => {
                let (_, child_rects) = stack_layout(self, given_size)?;
                child_rects
            }
        };
        Ok(res)
    }

    pub fn size(&self, available: Rect) -> Result<Point> {
        let res = match self.kind {
            LayoutKind::Panel { size, margin } => match size {
                PanelSize::Fit => {
                    let m = Point::new(margin, margin);
                    let inner_avail = available.sub_rect(m, available.size() - m * 2);
                    self.inner()?.size(inner_avail)? + m * 2
                }
                PanelSize::Fill => available.size(),
                PanelSize::Fixed(size) => size,
            },
            LayoutKind::Centered { dimension } => {
                let inner_size = self.inner()?.size(available)?;
                match dimension {
                    Some(Axis2D::X) => Point::new(available.size().x, inner_size.y),
                    Some(Axis2D::Y) => Point::new(inner_size.x, available.size().y),
                    None => available.size(),
                }
            }
            LayoutKind::Anchored { edge } => {
                let inner_size = self.inner()?.size(available)?;
                match edge {
                    AnchorEdge::Top => Point::new(available.size().x, inner_size.y),
                    AnchorEdge::Bottom => Point::new(available.size().x, inner_size.y),
                    AnchorEdge::Left => Point::new(inner_size.x, available.size().y),
                    AnchorEdge::Right => Point::new(inner_size.x, available.size().y),
                }
            }
            LayoutKind::Stack { .. } => {
                let (size, _) = stack_layout(self, available)?;
                size
            }
        };
        Ok(res)
    }

    fn _set_layout(&mut self, region: Rect) -> Result<()> {
        self.rect = region;
        let child_rects = self.child_rects(region)?;
        for (child, rect) in self.children.iter_mut().zip(child_rects) {
            child._set_layout(rect)?;
        }
        Ok(())
    }

    pub fn compute(mut self, available: Rect) -> Result<Self> {
        self._set_layout(Rect::with_size(available.min, self.size(available)?))?;
        Ok(self)
    }

    pub fn visit_data<F: LayoutVisitor<Data>>(&self, mut f: F) -> Result<()> {
        let mut to_process = vec![self];
        while let Some(elem) = to_process.pop() {
            if let Some(data) = &elem.data {
                f.call(data, elem);
            }
            for child in &elem.children {
                to_process.push(child);
            }
        }
        Ok(())
    }

    pub fn visit_mut<F: LayoutMutVisitor<Data>>(&mut self, mut f: F) {
        let mut to_process = vec![self];
        while let Some(elem) = to_process.pop() {
            f.call(elem);
            for child in &mut elem.children {
                to_process.push(child);
            }
        }
    }
}

pub fn panel<Data>(size: PanelSize, margin: i32) -> LayoutElement<Data> {
    LayoutElement::new(LayoutKind::Panel { size, margin })
}
pub fn centered<Data>(dimension: Option<Axis2D>) -> LayoutElement<Data> {
    LayoutElement::new(LayoutKind::Centered { dimension })
}
pub fn anchored<Data>(edge: AnchorEdge) -> LayoutElement<Data> {
    LayoutElement::new(LayoutKind::Anchored { edge })
}
pub fn stack<Data>(dimension: Axis2D) -> LayoutElement<Data> {
    LayoutElement::new(LayoutKind::Stack { dimension })
}
