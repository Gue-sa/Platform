use getset::{CloneGetters, Getters, Setters};
use shared::{common::types::VoyageStatus, voyage_order::VoyageOrder};

#[derive(Debug, Clone, Getters)]
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
    pub fn new(nav_status: u8, start_p: (u16, u16), end_p: (u16, u16), target_speed: u16) -> Self {
        let distance: u16 = ((start_p.0 as f64 - end_p.0 as f64).powf(2.)
            + (start_p.1 as f64 - end_p.1 as f64).powf(2.))
        .sqrt()
        .round() as u16;

        let heading: u16 = (end_p.0 as f64 - start_p.0 as f64)
            .atan2(end_p.1 as f64 - start_p.1 as f64)
            .to_degrees()
            .round() as u16;
        //let minutes_duration: u16 = distance / target_speed;

        Self {
            navigational_status: nav_status,
            start_point: start_p,
            end_point: end_p,
            distance: distance,
            //target_speed: target_speed,
            heading: heading,
            //minutes_duration: minutes_duration
        }
    }
}

impl Voyage {
    pub fn from(voyage_order: VoyageOrder, current_position: (u16, u16)) -> Self {
        let segment: VoyageSegment =
            VoyageSegment::new(0, current_position, *voyage_order.body().destination_position(), 0);

        Self {
            order: voyage_order,
            status: VoyageStatus::UnderRevision,
            segments: vec![segment],
            current_segment: 0,
        }
    }

    pub fn next_segment(&mut self) -> &VoyageSegment {
        self.current_segment += 1;
        &self.segments[self.current_segment]
    }
}
