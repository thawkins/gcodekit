#[derive(Clone, Debug, Default)]
pub struct MachinePosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub a: Option<f32>,
    pub b: Option<f32>,
    pub c: Option<f32>,
    pub d: Option<f32>,
}

impl MachinePosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            a: None,
            b: None,
            c: None,
            d: None,
        }
    }

    pub fn with_a(mut self, a: f32) -> Self {
        self.a = Some(a);
        self
    }

    pub fn with_b(mut self, b: f32) -> Self {
        self.b = Some(b);
        self
    }

    pub fn with_c(mut self, c: f32) -> Self {
        self.c = Some(c);
        self
    }

    pub fn with_d(mut self, d: f32) -> Self {
        self.d = Some(d);
        self
    }

    pub fn get_axis(&self, axis: char) -> Option<f32> {
        match axis {
            'X' | 'x' => Some(self.x),
            'Y' | 'y' => Some(self.y),
            'Z' | 'z' => Some(self.z),
            'A' | 'a' => self.a,
            'B' | 'b' => self.b,
            'C' | 'c' => self.c,
            'D' | 'd' => self.d,
            _ => None,
        }
    }

    pub fn set_axis(&mut self, axis: char, value: f32) {
        match axis {
            'X' | 'x' => self.x = value,
            'Y' | 'y' => self.y = value,
            'Z' | 'z' => self.z = value,
            'A' | 'a' => self.a = Some(value),
            'B' | 'b' => self.b = Some(value),
            'C' | 'c' => self.c = Some(value),
            'D' | 'd' => self.d = Some(value),
            _ => {}
        }
    }

    pub fn distance_to(&self, other: &MachinePosition) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    pub fn format(&self) -> String {
        let mut s = format!("X{:.3} Y{:.3} Z{:.3}", self.x, self.y, self.z);
        if let Some(a) = self.a {
            s.push_str(&format!(" A{:.3}", a));
        }
        if let Some(b) = self.b {
            s.push_str(&format!(" B{:.3}", b));
        }
        if let Some(c) = self.c {
            s.push_str(&format!(" C{:.3}", c));
        }
        if let Some(d) = self.d {
            s.push_str(&format!(" D{:.3}", d));
        }
        s
    }
}
