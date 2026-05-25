use getset::{CloneGetters, Getters, Setters};
use shared::{common::types::VoyageStatus, voyage_order::VoyageOrder};

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct VoyageSegment {
    navigational_status: u8,
    start_point: (u16, u16),
    end_point: (u16, u16),
    distance: u16,
    //target_speed: u16,
    heading: u16,
    //minutes_duration: u16
}

#[derive(Debug, Clone, Getters, CloneGetters, Setters)]
pub struct Voyage {
    #[getset(get = "pub", set = "pub")]
    order: VoyageOrder,
    #[getset(get = "pub", set = "pub")]
    status: VoyageStatus,
    #[getset(get_clone = "pub")]
    segments: Vec<VoyageSegment>,
    #[getset(get = "pub")]
    current_segment: usize,
    //distance: u16,
    //minutes_duration: u16
}

impl VoyageSegment {
    pub fn new(nav_status: u8, start_p: (u16, u16), end_p: (u16, u16), trgt_speed: u16) -> Self {
        let dist = ((start_p.0 as f64 - end_p.0 as f64).powf(2.)
            + (start_p.1 as f64 - end_p.1 as f64).powf(2.))
        .sqrt()
        .round() as u16;

        let heading = (end_p.0 as f64 - start_p.0 as f64)
            .atan2(end_p.1 as f64 - start_p.1 as f64)
            .to_degrees()
            .round() as u16;
        //let minutes_duration = distance / target_speed;

        Self {
            navigational_status: nav_status,
            start_point: start_p,
            end_point: end_p,
            distance: dist,
            //target_speed: target_speed,
            heading: heading,
            //minutes_duration: minutes_duration
        }
    }

    pub fn distance_from_end(&self, p: (u16, u16)) -> u16 {
        (((self.end_point.0 as i32 - p.0 as i32) ^ 2 + (self.end_point.1 as i32 - p.1 as i32) ^ 2)
            as f64)
            .sqrt() as u16
    }

    pub fn expected_lat(&self, lon: u16) -> u16 {
        ((self.end_point.1 as f64 - self.start_point.1 as f64)
            / (self.end_point.0 as f64 - self.start_point.0 as f64)
            * (lon as f64))
            .round() as u16
            + self.start_point.0
    }

    pub fn expected_lon(&self, lat: u16) -> u16 {
        ((self.end_point.0 as f64 - self.start_point.0 as f64)
            / (self.end_point.1 as f64 - self.start_point.1 as f64)
            * (lat as f64))
            .round() as u16
            + self.start_point.1
    }

    pub fn orthogonal_projection(&self, p: (u16, u16)) -> (u16, u16) {
        let a = self.end_point.1 as f64 - self.start_point.1 as f64;
        let b = self.start_point.0 as f64 - self.end_point.0 as f64;
        let c = self.end_point.0 as f64 * self.start_point.1 as f64
            - self.end_point.1 as f64 * self.start_point.0 as f64;

        let x =
            ((b * b * p.0 as f64 - a * b * p.1 as f64 - a * c) / (a * a + b * b)).round() as u16;
        let y =
            ((a * a * p.1 as f64 - a * b * p.0 as f64 - b * c) / (a * a + b * b)).round() as u16;

        (x, y)
    }

    pub fn distance_from_route(&self, p: (u16, u16)) -> f64 {
        let orth_proj = self.orthogonal_projection(p);
        let dist = ((orth_proj.0 as i32 - p.0 as i32).pow(2) as f64
            + (orth_proj.1 as i32 - p.1 as i32).pow(2) as f64)
            .sqrt();

        dist
    }
}

impl Voyage {
    pub fn from(voyage_order: VoyageOrder, current_pos: (u16, u16)) -> Self {
        let segment = VoyageSegment::new(
            0,
            current_pos,
            *voyage_order.body().destination_position(),
            0,
        );

        Self {
            order: voyage_order,
            status: VoyageStatus::UnderRevision,
            segments: vec![segment],
            current_segment: 0,
        }
    }

    pub fn next_segment(&mut self) -> Option<&VoyageSegment> {
        if self.segments.len() - 1 > self.current_segment {
            self.current_segment += 1;
            Some(&self.segments[self.current_segment])
        } else {
            None
        }
    }

    pub fn get_current_segment(&mut self) -> &VoyageSegment {
        &mut self.segments[self.current_segment]
    }
}
