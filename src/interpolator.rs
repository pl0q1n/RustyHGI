use image::GrayImage;


#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum InterpolationType {
    Crossed,
    Line,
    Previous,
}

pub trait Interpolator {
    fn interpolate(&self, levels: usize, level: usize, at: (u32, u32), image: &GrayImage) -> u8;
}

pub struct LeftTop; // almost no-op
impl Interpolator for LeftTop {
    #[inline(always)]
    fn interpolate(&self, levels: usize, level: usize, (x, y): (u32, u32), image: &GrayImage) -> u8 {
        let step = 1 << (levels - level + 1);
        let mask = step - 1;

        let x_top   = x - (x & mask); 
        let y_left  = y - (y & mask);

        use image::GenericImage;
        unsafe { image.unsafe_get_pixel(x_top, y_left).data[0] }        
    }
}

pub struct Crossed;

// Helper struct for Crossed interpolator
#[derive(Default)]
struct CrossedValues {
    left_top: u8,
    right_top: u8,
    left_bot: u8,
    right_bot: u8,
}

impl CrossedValues {
    #[inline(always)]
    pub fn prediction(&self) -> u8 {
        let average = |x, y| (x as usize + y as usize + 1) >> 1; // div 2

        let left  = average(self.left_top,  self.left_bot);
        let right = average(self.right_bot, self.right_top);
        let top   = average(self.right_top, self.left_top);
        let bot   = average(self.right_bot, self.left_bot);

        let average = (left + right + top + bot) >> 2; // div 4

        average as u8
    }
}

impl Interpolator for Crossed {
    #[inline(always)]
    fn interpolate(
        &self,
        levels: usize,
        level: usize,
        (x, y): (u32, u32), // column, line
        image: &GrayImage
    ) -> u8 {
        // step size on previous level
        let step = 1 << (levels - level + 1);
        let mask = step - 1;

        let x_top   = x - (x & mask); 
        let y_left  = y - (y & mask);
        let x_bot   = x_top + step;
        let y_right = y_left + step;

        let get_pixel = |x, y| {
            use image::GenericImage;
            if x < image.width() && y < image.height() {
                unsafe { image.unsafe_get_pixel(x, y).data[0] }
            } else {
                0
            }
        };

        CrossedValues {
            left_top:  get_pixel(x_top, y_left),
            right_top: get_pixel(x_top, y_right),
            left_bot:  get_pixel(x_bot, y_left),
            right_bot: get_pixel(x_bot, y_right)
        }.prediction()
    }
}