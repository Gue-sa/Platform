import { useState } from "react";
import { DropDownMenu } from "~/dropdown_menu/dropdown_menu";
import { VoyageOrderVersion } from "~/order_version/order_version";

export function VoyageOrder() {
    const [isOpened, setIsOpened] = useState(false);

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
                <p className="order-id">Ordre n°1148949846</p>
                <p className="current-version-number">Version n°5</p>
                <p className="executant">Exécutant : None</p>
                <p className="status">Non assigné</p>
            </div>
            <div className="voyage-order-body">
                <div className="current-version">
                    <ul>
                        <li>Date de création : 25/11/2005, 21h00</li>
                        <li>Statut : Non assigné</li>
                        <li>Exécutant : Aucun</li>
                        <li>Destination : Port de Dakar (x: 99, y: 99)</li>
                        <li>ETA : 25/09/2009, 15h00</li>
                        <li>Type de cargo : Hydrocarbures</li>
                        <li>Profil de vitesse : Eco</li>
                    </ul>
                </div>
                <DropDownMenu
                    subClassName="voyage-order-other-versions"
                    title="Autres versions"
                >
                    <VoyageOrderVersion />
                    <VoyageOrderVersion />
                    <VoyageOrderVersion />
                </DropDownMenu>
            </div>
        </div>
    );
}
