use image::{GrayImage, Luma};
use std::collections::HashSet;

/// Configuration for bitmap vectorization
#[derive(Clone, Debug)]
pub struct VectorizationConfig {
    pub threshold_method: ThresholdMethod,
    pub threshold_value: u8,
    pub noise_reduction: bool,
    pub smoothing: bool,
    pub min_contour_length: usize,
    pub simplification_tolerance: f32,
}

impl Default for VectorizationConfig {
    fn default() -> Self {
        Self {
            threshold_method: ThresholdMethod::Otsu,
            threshold_value: 128,
            noise_reduction: true,
            smoothing: true,
            min_contour_length: 10,
            simplification_tolerance: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ThresholdMethod {
    Fixed,
    Otsu,
    Adaptive,
}

/// Advanced bitmap processing and vectorization
pub struct BitmapProcessor;

impl BitmapProcessor {
    /// Vectorize a bitmap image to polylines using multiple algorithms
    pub fn vectorize_bitmap(img: &GrayImage, config: &VectorizationConfig) -> Vec<Vec<(f32, f32)>> {
        // Step 1: Preprocessing
        let processed = Self::preprocess_image(img, config);

        // Step 2: Thresholding
        let binary = Self::apply_thresholding(&processed, config);

        // Step 3: Contour tracing
        let contours = Self::trace_contours(&binary, config);

        // Step 4: Post-processing
        Self::postprocess_contours(contours, config)
    }

    /// Preprocess image with noise reduction and smoothing
    fn preprocess_image(img: &GrayImage, config: &VectorizationConfig) -> GrayImage {
        let mut processed = img.clone();

        if config.noise_reduction {
            processed = Self::apply_median_filter(&processed, 3);
        }

        if config.smoothing {
            processed = Self::apply_gaussian_blur(&processed, 1.0);
        }

        processed
    }

    /// Apply various thresholding methods
    fn apply_thresholding(img: &GrayImage, config: &VectorizationConfig) -> GrayImage {
        let mut binary = GrayImage::new(img.width(), img.height());

        match config.threshold_method {
            ThresholdMethod::Fixed => {
                for (x, y, pixel) in img.enumerate_pixels() {
                    let value = if pixel[0] < config.threshold_value {
                        0
                    } else {
                        255
                    };
                    binary.put_pixel(x, y, Luma([value]));
                }
            }
            ThresholdMethod::Otsu => {
                let threshold = Self::calculate_otsu_threshold(img);
                for (x, y, pixel) in img.enumerate_pixels() {
                    let value = if pixel[0] < threshold { 0 } else { 255 };
                    binary.put_pixel(x, y, Luma([value]));
                }
            }
            ThresholdMethod::Adaptive => {
                binary = Self::apply_adaptive_thresholding(img, 15, 10);
            }
        }

        binary
    }

    /// Calculate Otsu's threshold
    fn calculate_otsu_threshold(img: &GrayImage) -> u8 {
        let mut histogram = [0u32; 256];

        // Build histogram
        for pixel in img.pixels() {
            histogram[pixel[0] as usize] += 1;
        }

        let total_pixels = img.width() * img.height();
        let mut sum = 0.0;
        for i in 0..256 {
            sum += i as f32 * histogram[i] as f32;
        }

        let mut sum_b = 0.0;
        let mut w_b = 0.0;
        let mut max_variance = 0.0;
        let mut threshold = 0u8;

        for t in 0..256 {
            w_b += histogram[t] as f32;
            if w_b == 0.0 {
                continue;
            }

            let w_f = total_pixels as f32 - w_b;
            if w_f == 0.0 {
                break;
            }

            sum_b += t as f32 * histogram[t] as f32;

            let m_b = sum_b / w_b;
            let m_f = (sum - sum_b) / w_f;

            let variance = w_b * w_f * (m_b - m_f).powi(2);

            if variance > max_variance {
                max_variance = variance;
                threshold = t as u8;
            }
        }

        threshold
    }

    /// Apply adaptive thresholding using local mean
    fn apply_adaptive_thresholding(img: &GrayImage, block_size: u32, c: i32) -> GrayImage {
        let mut result = GrayImage::new(img.width(), img.height());
        let half_block = block_size / 2;

        for y in 0..img.height() {
            for x in 0..img.width() {
                let x_start = x.saturating_sub(half_block);
                let x_end = (x + half_block + 1).min(img.width());
                let y_start = y.saturating_sub(half_block);
                let y_end = (y + half_block + 1).min(img.height());

                let mut sum = 0u32;
                let mut count = 0u32;

                for by in y_start..y_end {
                    for bx in x_start..x_end {
                        sum += img.get_pixel(bx, by)[0] as u32;
                        count += 1;
                    }
                }

                let mean = (sum / count) as i32;
                let pixel_value = img.get_pixel(x, y)[0] as i32;
                let thresholded = if pixel_value < (mean - c) { 0 } else { 255 };

                result.put_pixel(x, y, Luma([thresholded as u8]));
            }
        }

        result
    }

    /// Apply median filter for noise reduction
    fn apply_median_filter(img: &GrayImage, size: u32) -> GrayImage {
        let mut result = GrayImage::new(img.width(), img.height());
        let half_size = size / 2;

        for y in 0..img.height() {
            for x in 0..img.width() {
                let mut values = Vec::new();

                for dy in -(half_size as i32)..=(half_size as i32) {
                    for dx in -(half_size as i32)..=(half_size as i32) {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < img.width() as i32 && ny >= 0 && ny < img.height() as i32
                        {
                            values.push(img.get_pixel(nx as u32, ny as u32)[0]);
                        }
                    }
                }

                values.sort();
                let median = values[values.len() / 2];
                result.put_pixel(x, y, Luma([median]));
            }
        }

        result
    }

    /// Apply Gaussian blur for smoothing
    fn apply_gaussian_blur(img: &GrayImage, sigma: f32) -> GrayImage {
        let kernel_size = (sigma * 3.0).ceil() as u32 * 2 + 1;
        let mut kernel = vec![vec![0.0; kernel_size as usize]; kernel_size as usize];

        // Generate Gaussian kernel
        let center = kernel_size as f32 / 2.0;
        let mut sum = 0.0;
        for y in 0..kernel_size {
            for x in 0..kernel_size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let value = (-(dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
                kernel[y as usize][x as usize] = value;
                sum += value;
            }
        }

        // Normalize kernel
        for y in 0..kernel_size {
            for x in 0..kernel_size {
                kernel[y as usize][x as usize] /= sum;
            }
        }

        // Apply convolution
        let mut result = GrayImage::new(img.width(), img.height());
        let half_kernel = kernel_size / 2;

        for y in 0..img.height() {
            for x in 0..img.width() {
                let mut sum = 0.0;

                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let px = x as i32 + kx as i32 - half_kernel as i32;
                        let py = y as i32 + ky as i32 - half_kernel as i32;

                        if px >= 0 && px < img.width() as i32 && py >= 0 && py < img.height() as i32
                        {
                            let pixel_value = img.get_pixel(px as u32, py as u32)[0] as f32;
                            sum += pixel_value * kernel[ky as usize][kx as usize];
                        }
                    }
                }

                result.put_pixel(x, y, Luma([sum as u8]));
            }
        }

        result
    }

    /// Trace contours using improved algorithm
    fn trace_contours(img: &GrayImage, config: &VectorizationConfig) -> Vec<Vec<(f32, f32)>> {
        let mut contours = Vec::new();
        let mut visited = HashSet::new();

        for y in 0..img.height() {
            for x in 0..img.width() {
                if img.get_pixel(x, y)[0] == 0 && !visited.contains(&(x, y)) {
                    // Found unvisited black pixel, start tracing
                    if let Some(contour) = Self::trace_single_contour(img, x, y, &mut visited)
                        && contour.len() >= config.min_contour_length {
                            contours.push(contour);
                        }
                }
            }
        }

        contours
    }

    /// Trace a single contour using Moore-Neighbor tracing
    fn trace_single_contour(
        img: &GrayImage,
        start_x: u32,
        start_y: u32,
        visited: &mut HashSet<(u32, u32)>,
    ) -> Option<Vec<(f32, f32)>> {
        let mut contour = Vec::new();
        let mut current = (start_x, start_y);
        let start = current;

        // Directions: 0=right, 1=down-right, 2=down, 3=down-left, 4=left, 5=up-left, 6=up, 7=up-right
        let directions = [
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        let mut direction = 0; // Start by looking right

        loop {
            visited.insert(current);
            contour.push((current.0 as f32, current.1 as f32));

            // Look for next pixel in contour
            let mut found = false;
            for i in 0..8 {
                let check_dir = (direction + i) % 8;
                let (dx, dy) = directions[check_dir];
                let nx = current.0 as i32 + dx;
                let ny = current.1 as i32 + dy;

                if nx >= 0 && nx < img.width() as i32 && ny >= 0 && ny < img.height() as i32 {
                    let nx_u = nx as u32;
                    let ny_u = ny as u32;

                    if img.get_pixel(nx_u, ny_u)[0] == 0 && !visited.contains(&(nx_u, ny_u)) {
                        current = (nx_u, ny_u);
                        direction = (check_dir + 6) % 8; // Turn left for next search
                        found = true;
                        break;
                    }
                }
            }

            if !found || current == start {
                break;
            }
        }

        if contour.len() >= 3 {
            Some(contour)
        } else {
            None
        }
    }

    /// Post-process contours with simplification and smoothing
    fn postprocess_contours(
        contours: Vec<Vec<(f32, f32)>>,
        config: &VectorizationConfig,
    ) -> Vec<Vec<(f32, f32)>> {
        contours
            .into_iter()
            .map(|contour| Self::simplify_contour(contour, config.simplification_tolerance))
            .collect()
    }

    /// Simplify a contour using Douglas-Peucker algorithm
    fn simplify_contour(contour: Vec<(f32, f32)>, tolerance: f32) -> Vec<(f32, f32)> {
        if contour.len() <= 2 {
            return contour;
        }

        let mut simplified = Vec::new();

        // Find the point with maximum distance
        let mut max_distance = 0.0;
        let mut max_index = 0;

        let start = contour[0];
        let end = contour[contour.len() - 1];

        for (i, &point) in contour.iter().enumerate().skip(1) {
            if i == contour.len() - 1 {
                continue;
            }

            let distance = Self::point_to_line_distance(point, start, end);
            if distance > max_distance {
                max_distance = distance;
                max_index = i;
            }
        }

        // If max distance is greater than tolerance, recursively simplify
        if max_distance > tolerance {
            let left = Self::simplify_contour(contour[0..=max_index].to_vec(), tolerance);
            let right = Self::simplify_contour(contour[max_index..].to_vec(), tolerance);

            // Combine results, avoiding duplicate middle point
            simplified.extend(left);
            simplified.extend_from_slice(&right[1..]);
        } else {
            simplified.push(start);
            simplified.push(end);
        }

        simplified
    }

    /// Calculate distance from point to line segment
    fn point_to_line_distance(
        point: (f32, f32),
        line_start: (f32, f32),
        line_end: (f32, f32),
    ) -> f32 {
        let (px, py) = point;
        let (x1, y1) = line_start;
        let (x2, y2) = line_end;

        let dx = x2 - x1;
        let dy = y2 - y1;

        if dx == 0.0 && dy == 0.0 {
            return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
        }

        let t = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
        let t = t.max(0.0).min(1.0);

        let closest_x = x1 + t * dx;
        let closest_y = y1 + t * dy;

        ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
    }
}
