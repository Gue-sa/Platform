export interface VoyageOrderResult {
    id: number;
    creation_date: string;
    status: number;
    executant: number | null;
    current_version_number: number;
}

export interface VoyageVersionResult {
    id: number;
    version_number: number;
    creation_date: string;
    order_id: number;
    destination_id: number;
    eta: string;
    cargo_type: number;
    speed_profile: number;
}

export interface DestinationResult {
    id: number;
    name: string;
    longitude: number;
    latitude: number;
}

export interface BoatStaticData {
    mmsi: number;
    imo_number: number;
    call_sign: string;
    name: string;
    type_of_ship_and_cargo_type: number;
    position_accuracy: number;
    ais_version: number;
    type_of_epf_device: number;
    a: number;
    b: number;
    c: number;
    d: number;
    spare: number;
}

export interface BoatVoyageData {
    destination: string;
    eta_month: number;
    eta_day: number;
    eta_hour: number;
    eta_minute: number;
    maximum_present_static_draught: number;
    dte: number;
    raim_flag: number;
}

export interface BoatNavigationData {
    navigational_status: number;
    time_stamp: number;
    special_maneuvre_indicator: number;
    latitude: number;
    longitude: number;
    course_over_ground: number;
    speed_over_ground: number;
    rate_of_turn: number;
    true_heading: number;
}

export interface BoatInfoResult {
    static_data: BoatStaticData;
    voyage_data: BoatVoyageData;
    navigation_data: BoatNavigationData;
}

export type VoyageResult = [
    VoyageOrderResult,
    VoyageVersionResult,
    DestinationResult,
][];

export type DestinationsResult = DestinationResult[];

export type VersionResult = [VoyageVersionResult, DestinationResult][];

export type BoatInfoRegistry = [number, BoatInfoResult][];
