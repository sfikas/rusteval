#[derive(Debug)]
pub struct Point {
	x: Option<u32>,
	y: Option<u32>,
}

impl Point {
	fn new() -> Point { 
		Point { x: None, y: None }
	}
	pub fn get_x(&self) -> u32 {
		match self.x {
			Some(v) => v,
			None 	=> panic!("Undefined value")
		}
	}
	pub fn get_y(&self) -> u32 {
		match self.y {
			Some(v) => v,
			None 	=> panic!("Undefined value")
		}
	}
}

#[derive(Debug)]
pub struct Rectangle {
	pub min: Point,
	pub max: Point,
}

impl Rectangle {
	pub fn width(&self) -> u32 { self.max.get_x() - self.min.get_x() + 1 }
	pub fn height(&self)-> u32 { self.max.get_y() - self.min.get_y() + 1 }
	fn area(&self)  -> u32 { self.width()*self.height() }
	pub fn is_ok(&self) -> bool { self.max.get_x() > self.min.get_x() && self.max.get_y() > self.min.get_y() }

	pub fn new() -> Rectangle { Rectangle { min: Point::new(), max: Point::new() } }
	pub fn set_min(&mut self, x: u32, y: u32) -> &mut Rectangle { 
		if let (Some(max_x), Some(max_y)) = (self.max.x, self.max.y) {
			if max_x <= x || max_y <= y { panic!("Rectangle dimensions non-positive!"); }
		}
		self.min.x = Some(x); 
		self.min.y = Some(y);		
		self
	}
	pub fn set_max(&mut self, x: u32, y: u32) -> &mut Rectangle { 
		if let (Some(min_x), Some(min_y)) = (self.min.x, self.min.y) {
			if min_x >= x || min_y >= y { panic!("Rectangle dimensions non-positive!"); }
		}		
		self.max.x = Some(x);
		self.max.y = Some(y);		
		self
	}
	pub fn set_size(&mut self, w: u32, h: u32) -> &mut Rectangle { 
		self.max.x = Some(self.min.get_x() + w - 1);
		self.max.y = Some(self.min.get_y() + h - 1);
		self
	}
}

#[test]
fn test_rectangle_and_point() {
	let mut a = Rectangle::new();
	a.set_min(1, 1).set_size(10, 10);
	assert_eq!(a.width(), 10);
	assert_eq!(a.height(), 10);
	assert_eq!(a.area(), 100);
}

#[test]
#[should_panic]
fn test_bad_rectangle() {
	let mut a = Rectangle::new();
	a.set_min(50, 50).set_max(30, 40);
}