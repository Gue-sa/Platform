import { useState } from "react";
import type { BoatInfoRegistry } from "~/types";

interface MetricsProps {
    boats_info: BoatInfoRegistry;
}

export function Metrics({ boats_info }: MetricsProps) {
    const [selectedBoat, setSelectedBoat] = useState(-1);

    const selectedBoatInfo = boats_info.find(([id]) => id == selectedBoat)?.[1];

    return (
        <div className="metrics-screen">
            <div className="metrics-category vessels-list-container">
                <p className="metrics-category-title">Flotte</p>
                <ul>
                    {boats_info.map((boat_info) => (
                        <li
                            key={boat_info[1].static_data.mmsi}
                            className={
                                selectedBoat == boat_info[1].static_data.mmsi
                                    ? "boat-mmsi selected"
                                    : "boat-mmsi"
                            }
                            onClick={() => {
                                setSelectedBoat(boat_info[1].static_data.mmsi);
                            }}
                        >
                            {boat_info[1].static_data.mmsi}
                        </li>
                    ))}
                </ul>
            </div>
            <div className="metrics-category boat-info-container">
                <p className="metrics-category-title">Informations bateau</p>
                {selectedBoat != -1 ? (
                    <>
                        <p className="boat-info-category-title">
                            Données statiques
                        </p>
                        <ul>
                            <li>MMSI : {selectedBoatInfo?.static_data.mmsi}</li>
                            <li>
                                IMO : {selectedBoatInfo?.static_data.imo_number}
                            </li>
                            <li>
                                Callsign :{" "}
                                {selectedBoatInfo?.static_data.call_sign}
                            </li>
                            <li>Nom : {selectedBoatInfo?.static_data.name}</li>
                            <li>
                                Type de bateau / cargo :{" "}
                                {
                                    selectedBoatInfo?.static_data
                                        .type_of_ship_and_cargo_type
                                }
                            </li>
                            <li>
                                Précision du positionnement :{" "}
                                {
                                    selectedBoatInfo?.static_data
                                        .position_accuracy
                                }
                            </li>
                            <li>
                                Version de l'AIS :{" "}
                                {selectedBoatInfo?.static_data.ais_version}
                            </li>
                            <li>
                                Type d'équipement EPF :{" "}
                                {
                                    selectedBoatInfo?.static_data
                                        .type_of_epf_device
                                }
                            </li>
                            <li>A : {selectedBoatInfo?.static_data.a}</li>
                            <li>B : {selectedBoatInfo?.static_data.b}</li>
                            <li>C : {selectedBoatInfo?.static_data.c}</li>
                            <li>D : {selectedBoatInfo?.static_data.d}</li>
                        </ul>
                        <p className="boat-info-category-title">Voyage</p>
                        <ul>
                            <li>
                                Destination :{" "}
                                {selectedBoatInfo?.voyage_data.destination}
                            </li>
                            <li>
                                ETA : {selectedBoatInfo?.voyage_data.eta_day}/
                                {selectedBoatInfo?.voyage_data.eta_month},{" "}
                                {selectedBoatInfo?.voyage_data.eta_hour}h
                                {selectedBoatInfo?.voyage_data.eta_minute}
                            </li>
                            <li>
                                Tirant d'eau statique maximal :{" "}
                                {
                                    selectedBoatInfo?.voyage_data
                                        .maximum_present_static_draught
                                }
                            </li>
                            <li>DTE : {selectedBoatInfo?.voyage_data.dte}</li>
                            <li>
                                RAIM : {selectedBoatInfo?.voyage_data.raim_flag}
                            </li>
                        </ul>
                        <p className="boat-info-category-title">Navigation</p>
                        <ul>
                            <li>
                                Statut de navigation :{" "}
                                {
                                    selectedBoatInfo?.navigation_data
                                        .navigational_status
                                }
                            </li>
                            <li>
                                Timestamp :{" "}
                                {selectedBoatInfo?.navigation_data.time_stamp}
                            </li>
                            <li>
                                Indicateur de manoeuvre spéciale :{" "}
                                {
                                    selectedBoatInfo?.navigation_data
                                        .special_maneuvre_indicator
                                }
                            </li>
                            <li>
                                Latitude :{" "}
                                {selectedBoatInfo?.navigation_data.latitude}
                            </li>
                            <li>
                                Longitude :{" "}
                                {selectedBoatInfo?.navigation_data.longitude}
                            </li>
                            <li>
                                Route sur fond :{" "}
                                {
                                    selectedBoatInfo?.navigation_data
                                        .course_over_ground
                                }
                            </li>
                            <li>
                                Vitesse sur fond :{" "}
                                {
                                    selectedBoatInfo?.navigation_data
                                        .speed_over_ground
                                }{" "}
                                kt
                            </li>
                            <li>
                                Taux de virage :{" "}
                                {selectedBoatInfo?.navigation_data.rate_of_turn}{" "}
                                °/s
                            </li>
                            <li>
                                Cap vrai :{" "}
                                {selectedBoatInfo?.navigation_data.true_heading}
                                °
                            </li>
                        </ul>
                    </>
                ) : (
                    <p>
                        <i>
                            Cliquer sur un bateau pour obtenir les informations
                            sur ce dernier
                        </i>
                    </p>
                )}
            </div>
            <div className="metrics-category statistics-container">
                <p className="metrics-category-title">Statistiques</p>
                <ul>
                    <li>Taille de la flotte : 9</li>
                    <li>Bateaux en activité : 5</li>
                    <li>Nombre d'ordres de voyage : 1</li>
                    <li>Taux de complétion : 0 %</li>
                </ul>
            </div>
        </div>
    );
}
