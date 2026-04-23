import { useQuery } from "@tanstack/react-query";
import type { DestinationsResult } from "~/types";

const fetchDestinations = async (): Promise<DestinationsResult> => {
    const res = await fetch("http://localhost:8000/get_destinations");
    return res.json();
};

export default function VoyageOrderCreationForm() {
    const { data, isLoading, error } = useQuery({
        queryKey: ["destinations"],
        queryFn: () => fetchDestinations(),
        refetchInterval: 5000,
    });

    const handleSubmit = async (e: React.SubmitEvent<HTMLFormElement>) => {
        e.preventDefault();

        const formData = new FormData(e.currentTarget);

        const payload = {
            destination_id: Number(formData.get("destination-input")),
            ship_type: Number(formData.get("ship-and-cargo-type-input")),
            speed_profile: Number(formData.get("speed-profile-input")),
        };

        try {
            const res = await fetch("http://localhost:8000/add_voyage_order", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(payload),
            });

            if (res.ok) {
                alert("Ordre de voyage créé avec succès !");
            } else {
                alert("Erreur lors de la création");
            }
        } catch (err) {
            alert("Erreur réseau : " + err);
        }
    };

    return (
        <div className="voyage-order-creation-form-container">
            <form
                className="voyage-order-creation-form"
                onSubmit={handleSubmit}
            >
                <label htmlFor="destination-input">Destination</label>
                <select name="destination-input" required>
                    <option value="" disabled selected>
                        Sélectionnez une destination
                    </option>

                    {data?.map((d) => (
                        <option value={d.id}>
                            {d.name} : (x: {d.longitude}, y: {d.latitude})
                        </option>
                    ))}
                </select>

                <label htmlFor="ship-and-cargo-type-input">
                    Type de bateau et de cargo
                </label>
                <select name="ship-and-cargo-type-input" required>
                    <option value="" disabled selected>
                        Sélectionnez un type de bateau / cargo
                    </option>
                    <optgroup label="Navires à usage spécial">
                        <option value={1}>
                            01 - Navire scientifique / de recherche
                        </option>
                        <option value={2}>02 - Navire-école</option>
                        <option value={3}>
                            03 - Navire appartenant ou exploité par un
                            gouvernement
                        </option>
                        <option value={4}>04 - Brise-glace</option>
                        <option value={5}>
                            05 - Baliseur (Aides à la navigation)
                        </option>
                        <option value={6}>06 - Câblier</option>
                        <option value={7}>07 - Poseur de canalisations</option>
                        <option value={9}>
                            09 - Navire à usage spécial, aucune information
                            supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Navires de soutien">
                        <option value={11}>
                            11 - Unité flottante de production, de stockage et
                            de déchargement (FPSO)
                        </option>
                        <option value={12}>12 - Navire-usine</option>
                        <option value={13}>
                            13 - Navire de soutien à la pisciculture
                        </option>
                        <option value={14}>
                            14 - Navire de soutien offshore, etc.
                        </option>
                        <option value={17}>17 - Navire de construction</option>
                        <option value={18}>
                            18 - Bateau de relève d'équipage
                        </option>
                        <option value={19}>
                            19 - Navire de soutien, aucune information
                            supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Navires à effet de surface (WIG)">
                        <option value={20}>
                            20 - WIG, tous les navires de ce type
                        </option>
                        <option value={21}>
                            21 - WIG, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie X
                        </option>
                        <option value={22}>
                            22 - WIG, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie Y
                        </option>
                        <option value={23}>
                            23 - WIG, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie Z
                        </option>
                        <option value={24}>
                            24 - WIG, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie OS
                        </option>
                        <option value={29}>
                            29 - WIG, aucune information supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Embarcations spéciales (1)">
                        <option value={30}>30 - Navire de pêche</option>
                        <option value={31}>31 - Remorqueur</option>
                        <option value={32}>32 - Remorqueur</option>
                        <option value={33}>33 - Drague</option>
                        <option value={34}>34 - Navire de plongée</option>
                        <option value={35}>
                            35 - Navire de guerre ou auxiliaire naval
                        </option>
                        <option value={36}>36 - Voilier</option>
                        <option value={37}>
                            37 - Navire de plaisance à moteur
                        </option>
                        <option value={38}>38 - Chalutier</option>
                        <option value={39}>39 - Patrouilleur</option>
                    </optgroup>
                    <optgroup label="Navires à grande vitesse (HSC)">
                        <option value={40}>
                            40 - HSC, tous les navires de ce type
                        </option>
                        <option value={41}>
                            41 - HSC, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie X
                        </option>
                        <option value={42}>
                            42 - HSC, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie Y
                        </option>
                        <option value={43}>
                            43 - HSC, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie Z
                        </option>
                        <option value={44}>
                            44 - HSC, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie OS
                        </option>
                        <option value={45}>
                            45 - HSC, transportant des passagers
                        </option>
                        <option value={46}>
                            46 - HSC Ro-Ro (véhicules / fret ferroviaire)
                        </option>
                        <option value={49}>
                            49 - HSC, aucune information supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Embarcations spéciales (2)">
                        <option value={50}>50 - Bateau-pilote</option>
                        <option value={51}>
                            51 - Navires de recherche et de sauvetage (SAR)
                        </option>
                        <option value={52}>52 - Remorqueurs</option>
                        <option value={53}>
                            53 - Navires de ravitaillement portuaires ou de
                            pêche
                        </option>
                        <option value={54}>
                            54 - Navire d'intervention antipollution ou
                            anti-incendie
                        </option>
                        <option value={55}>
                            55 - Navires de maintien de l'ordre
                        </option>
                        <option value={56}>
                            56 - Libre 1 – pour attribution aux navires locaux
                        </option>
                        <option value={57}>
                            57 - Libre 2 – pour attribution aux navires locaux
                        </option>
                        <option value={58}>
                            58 - Transports sanitaires (Conventions de Genève
                            1949)
                        </option>
                        <option value={59}>
                            59 - Navires d'États non parties à un conflit armé
                        </option>
                    </optgroup>
                    <optgroup label="Navires à passagers">
                        <option value={60}>
                            60 - Navires à passagers, tous les navires de ce
                            type
                        </option>
                        <option value={61}>
                            61 - Navires à passagers, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie X
                        </option>
                        <option value={62}>
                            62 - Navires à passagers, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie Y
                        </option>
                        <option value={63}>
                            63 - Navires à passagers, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie Z
                        </option>
                        <option value={64}>
                            64 - Navires à passagers, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie OS
                        </option>
                        <option value={65}>
                            65 - Navire à passagers (croisière)
                        </option>
                        <option value={66}>
                            66 - Navire à passagers (ferry)
                        </option>
                        <option value={67}>
                            67 - Navire à passagers (excursion)
                        </option>
                        <option value={69}>
                            69 - Navires à passagers, aucune information
                            supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Navires de charge (Cargos)">
                        <option value={70}>
                            70 - Cargos, tous les navires de ce type
                        </option>
                        <option value={71}>
                            71 - Cargos, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie X
                        </option>
                        <option value={72}>
                            72 - Cargos, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie Y
                        </option>
                        <option value={73}>
                            73 - Cargos, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie Z
                        </option>
                        <option value={74}>
                            74 - Cargos, transportant marchandises dangereuses
                            (DG/MHB/HS/MP), catégorie OS
                        </option>
                        <option value={75}>75 - Cargo, vraquier</option>
                        <option value={76}>76 - Cargo, porte-conteneurs</option>
                        <option value={77}>77 - Cargo, roulier (Ro-Ro)</option>
                        <option value={78}>
                            78 - Cargo, chaland de débarquement
                        </option>
                        <option value={79}>
                            79 - Cargos, aucune information supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Navires-citernes">
                        <option value={80}>
                            80 - Navires-citernes, tous les navires de ce type
                        </option>
                        <option value={81}>
                            81 - Navires-citernes, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie X
                        </option>
                        <option value={82}>
                            82 - Navires-citernes, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie Y
                        </option>
                        <option value={83}>
                            83 - Navires-citernes, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie Z
                        </option>
                        <option value={84}>
                            84 - Navires-citernes, transportant marchandises
                            dangereuses (DG/MHB/HS/MP), catégorie OS
                        </option>
                        <option value={85}>
                            85 - Navire-citerne, transporteur de produits non
                            dangereux ou non polluants
                        </option>
                        <option value={86}>
                            86 - Remorqueur et barge-citerne articulés /
                            intégrés
                        </option>
                        <option value={89}>
                            89 - Navires-citernes, aucune information
                            supplémentaire
                        </option>
                    </optgroup>
                    <optgroup label="Autres types de navires">
                        <option value={90}>90 - Autres types de navires</option>
                        <option value={91}>
                            91 - Autres types de navires, transportant
                            marchandises dangereuses (DG/MHB/HS/MP), catégorie X
                        </option>
                        <option value={92}>
                            92 - Autres types de navires, transportant
                            marchandises dangereuses (DG/MHB/HS/MP), catégorie Y
                        </option>
                        <option value={93}>
                            93 - Autres types de navires, transportant
                            marchandises dangereuses (DG/MHB/HS/MP), catégorie Z
                        </option>
                        <option value={94}>
                            94 - Autres types de navires, transportant
                            marchandises dangereuses (DG/MHB/HS/MP), catégorie
                            OS
                        </option>
                        <option value={99}>
                            99 - Autres types de navires, aucune information
                            supplémentaire
                        </option>
                    </optgroup>
                </select>

                <label htmlFor="speed-profile-input">Profil de vitesse</label>
                <select name="speed-profile-input" required>
                    <option value="" disabled selected>
                        Sélectionnez un profil
                    </option>
                    <option value={0}>Economique</option>
                    <option value={1}>Classique</option>
                    <option value={2}>Rapide</option>
                </select>

                <input type="submit" value="Valider" />
            </form>
        </div>
    );
}
