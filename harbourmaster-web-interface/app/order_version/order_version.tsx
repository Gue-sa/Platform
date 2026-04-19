import { DropDownMenu } from "~/dropdown_menu/dropdown_menu";
import type { DestinationResult, VoyageVersionResult } from "~/types";

interface VoyageOrderVersionProps {
    version_data: VoyageVersionResult;
    destination_data: DestinationResult;
}

export function VoyageOrderVersion({
    version_data,
    destination_data,
}: VoyageOrderVersionProps) {
    return (
        <DropDownMenu
            subClassName="voyage-order-version"
            title={`Version n°${version_data.version_number}`}
        >
            <ul>
                <li>Date de création : {version_data.creation_date}</li>
                <li>
                    Destination : {destination_data.name} (x:{" "}
                    {destination_data.longitude}, y: {destination_data.latitude}
                    )
                </li>
                <li>ETA : {version_data.eta}</li>
                <li>Type de cargo : {version_data.cargo_type}</li>
                <li>Profil de vitesse : {version_data.speed_profile}</li>
            </ul>
        </DropDownMenu>
    );
}
