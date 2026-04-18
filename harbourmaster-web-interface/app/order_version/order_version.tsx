import { DropDownMenu } from "~/dropdown_menu/dropdown_menu";

export function VoyageOrderVersion() {
    return (
        <DropDownMenu subClassName="voyage-order-version" title="Version n°5">
            <ul>
                <li>Date de création : 25/11/2005, 21h00</li>
                <li>Statut : Non assigné</li>
                <li>Exécutant : Aucun</li>
                <li>Destination : Port de Dakar (x: 99, y: 99)</li>
                <li>ETA : 25/09/2009, 15h00</li>
                <li>Type de cargo : Hydrocarbures</li>
                <li>Profil de vitesse : Eco</li>
            </ul>
        </DropDownMenu>
    );
}
