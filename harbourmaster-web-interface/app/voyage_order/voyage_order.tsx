import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { DropDownMenu } from "~/dropdown_menu/dropdown_menu";
import { VoyageOrderVersion } from "~/order_version/order_version";
import type {
    DestinationResult,
    VersionResult,
    VoyageOrderResult,
    VoyageVersionResult,
} from "~/types";

interface VoyageOrderProps {
    order_data: VoyageOrderResult;
    current_version_data: VoyageVersionResult;
    destination_data: DestinationResult;
}

const fetchVoyageOrderVersions = async (
    order_id: number,
): Promise<VersionResult> => {
    const res = await fetch(
        `http://localhost:8000/get_voyage_order_versions?order_id=${order_id}`,
    );
    return res.json();
};

export function VoyageOrder({
    order_data,
    current_version_data,
    destination_data,
}: VoyageOrderProps) {
    const [isOpened, setIsOpened] = useState(false);

    const { data, isLoading, error } = useQuery({
        queryKey: ["voyage_order_versions", order_data.id],
        queryFn: () => fetchVoyageOrderVersions(order_data.id),
        refetchInterval: 5000,
        enabled: isOpened,
    });

    const otherVersions = Array.isArray(data)
        ? data.filter(([version]) => version.id !== current_version_data.id)
        : [];

    return (
        <div className={`voyage-order ${isOpened ? "opened" : "closed"}`}>
            <div
                title={`Cliquer pour ${isOpened ? "réduire" : "développer"}`}
                className="voyage-order-header"
                onClick={() => {
                    if (isOpened) {
                        setIsOpened(false);
                    } else {
                        setIsOpened(true);
                    }
                }}
            >
                <p className="order-id">Ordre n°{order_data.id}</p>
                <p className="current-version-number">
                    Version n°{order_data.current_version_number}
                </p>
                <p className="executant">Exécutant : {order_data.executant}</p>
                <p className="status">{order_data.status}</p>
            </div>
            <div className="voyage-order-body">
                <div className="current-version">
                    <ul>
                        <li>
                            Date de création :{" "}
                            {current_version_data.creation_date}
                        </li>
                        <li>
                            Destination : {destination_data.name} (x:{" "}
                            {destination_data.longitude}, y:{" "}
                            {destination_data.latitude})
                        </li>
                        <li>ETA : {current_version_data.eta}</li>
                        <li>
                            Type de cargo : {current_version_data.cargo_type}
                        </li>
                        <li>
                            Profil de vitesse :{" "}
                            {current_version_data.speed_profile}
                        </li>
                    </ul>
                </div>
                {otherVersions.length > 0 ? (
                    <DropDownMenu
                        subClassName="voyage-order-other-versions"
                        title={`Autres versions (${otherVersions.length})`}
                    >
                        {otherVersions.map(([version, destination]) =>
                            version.id != current_version_data.id ? (
                                <VoyageOrderVersion
                                    version_data={version}
                                    destination_data={destination}
                                />
                            ) : (
                                <></>
                            ),
                        )}
                    </DropDownMenu>
                ) : (
                    <></>
                )}
            </div>
        </div>
    );
}
