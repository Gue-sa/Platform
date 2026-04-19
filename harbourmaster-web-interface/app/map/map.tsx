import React from "react";

export interface MapData {
    mmsi: number;
    name: string;
    x: number; // 0 - 1920
    y: number; // 0 - 1080
    heading: number; // 0 - 360
    destX: number; // 0 - 1920
    destY: number; // 0 - 1080
}

export function Map({ ships }: { ships: MapData[] }) {
    return (
        <div
            style={{
                display: "flex",
                justifyContent: "center",
                alignItems: "center",
                overflow: "hidden",
            }}
        >
            <div
                style={{
                    position: "relative",
                    width: "100%",
                    height: "auto",
                    maxHeight: "100%",
                    aspectRatio: "16 / 9",
                    backgroundColor: "#0f172a",
                    backgroundImage:
                        "linear-gradient(#1e293b 1px, transparent 1px), linear-gradient(90deg, #1e293b 1px, transparent 1px)",
                    backgroundSize: "40px 40px",
                    overflow: "hidden",
                    boxShadow: "0 10px 25px rgba(0, 0, 0, 0.5)",
                    border: "1px solid #334155",
                }}
            >
                <svg
                    viewBox="0 0 1920 1080"
                    style={{
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: "100%",
                        height: "100%",
                        zIndex: 1,
                        pointerEvents: "none",
                    }}
                >
                    {ships.map((ship) => {
                        const shipColor = `hsl(${(ship.mmsi * 137) % 360}, 85%, 55%)`;
                        return (
                            <line
                                key={`line-${ship.mmsi}`}
                                x1={ship.x}
                                y1={1080 - ship.y}
                                x2={ship.destX}
                                y2={1080 - ship.destY}
                                stroke={shipColor}
                                strokeWidth="3"
                                strokeDasharray="12, 12"
                                opacity="0.6"
                            />
                        );
                    })}
                </svg>

                {ships.map((ship) => {
                    const shipColor = `hsl(${(ship.mmsi * 137) % 360}, 85%, 55%)`;

                    return (
                        <React.Fragment key={`markers-${ship.mmsi}`}>
                            <div
                                style={{
                                    position: "absolute",
                                    left: `${(ship.destX / 1920) * 100}%`,
                                    top: `${((1080 - ship.destY) / 1080) * 100}%`,
                                    transform: "translate(-50%, -50%)",
                                    width: "1.5vh",
                                    height: "1.5vh",
                                    backgroundColor: shipColor,
                                    borderRadius: "50%",
                                    border: "3px solid #0f172a",
                                    boxShadow: `0 0 10px ${shipColor}`,
                                    zIndex: 2,
                                }}
                                title={`Destination du navire ${ship.mmsi}: ${Math.round(ship.destX)}, ${Math.round(ship.destY)}`}
                            />

                            <div
                                style={{
                                    position: "absolute",
                                    left: `${(ship.x / 1920) * 100}%`,
                                    top: `${((1080 - ship.y) / 1080) * 100}%`,
                                    transform: `translate(-50%, -50%) rotate(${ship.heading}deg)`,
                                    width: "0",
                                    height: "0",
                                    borderLeft: "12px solid transparent",
                                    borderRight: "12px solid transparent",
                                    borderBottom: `35px solid ${shipColor}`,
                                    filter: `drop-shadow(0px 0px 8px ${shipColor})`,
                                    zIndex: 3,
                                    transition:
                                        "left 0.5s linear, top 0.5s linear, transform 0.5s ease-out",
                                }}
                                title={`Navire ${ship.mmsi} : ${Math.round(ship.x)}, ${Math.round(ship.y)}`}
                            />
                        </React.Fragment>
                    );
                })}
            </div>
        </div>
    );
}
