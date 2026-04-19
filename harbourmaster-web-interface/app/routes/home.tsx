import { Metrics } from "~/metrics/metrics";
import type { Route } from "./+types/home";
import { VoyageOrders } from "~/orders/orders";
import { Map } from "~/map/map";
import VoyageOrderCreationForm from "~/voyage_order_creation_form/voyage_order_creation_form";
import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import type { BoatInfoRegistry, VoyageResult } from "~/types";

const fetchVoyageOrders = async (): Promise<VoyageResult> => {
    const res = await fetch("http://localhost:8000/get_voyage_orders");
    return res.json();
};

const fetchBoatInfoRegistry = async (): Promise<BoatInfoRegistry> => {
    const res = await fetch("http://localhost:8000/get_boats_list");
    return res.json();
};

export function meta({}: Route.MetaArgs) {
    return [
        { title: "Interface Armateur" },
        {
            name: "description",
            content: "Interface armateur maquette plateforme maritime.",
        },
    ];
}

export default function Home() {
    const [isFormShown, setIsFormShown] = useState(false);

    const {
        data: voyageData,
        isLoading: _1,
        error: _2,
    } = useQuery({
        queryKey: ["voyage_orders"],
        queryFn: fetchVoyageOrders,
        refetchInterval: 5000,
    });

    const {
        data: boatsInfoData,
        isLoading: _3,
        error: _4,
    } = useQuery({
        queryKey: ["boats_list"],
        queryFn: fetchBoatInfoRegistry,
        refetchInterval: 5000,
    });

    const mapData = useMemo(() => {
        if (!boatsInfoData) return [];

        return boatsInfoData.map(([mmsi, info]) => {
            const activeOrder = voyageData?.find(
                (order) => order[0].executant == mmsi,
            );

            return {
                mmsi: mmsi,
                name: info.static_data.name,
                x: info.navigation_data.longitude,
                y: info.navigation_data.latitude,
                heading: info.navigation_data.true_heading,
                destX: activeOrder?.[2].longitude ?? info.navigation_data.longitude,
                destY: activeOrder?.[2].latitude ?? info.navigation_data.latitude,
            };
        });
    }, [boatsInfoData, voyageData]);

    return (
        <div className="harbourmaster-interface">
            <header>
                <h1 className="main-title">Interface Armateur</h1>
            </header>
            <main>
                <section className="screen-section left-screen-section">
                    <article className="map-article">
                        <p className="article-title map-article-title">Carte</p>
                        <Map ships={mapData} />
                    </article>
                    <article className="metrics-article">
                        <p className="article-title metrics-article-title">
                            Données
                        </p>
                        <Metrics boats_info={boatsInfoData ?? []} />
                    </article>
                </section>
                <section className="screen-section right-screen-section">
                    <article className="voyage-orders-article">
                        <p className="article-title voyage-orders-article-title">
                            Ordres de Voyage
                        </p>
                        <VoyageOrders voyage_orders={voyageData ?? []} />
                    </article>

                    <div className="add-voyage-order-button-container">
                        <button
                            className="add-voyage-order-button"
                            type="button"
                            onClick={() => setIsFormShown(true)}
                        >
                            Créer un ordre de voyage manuellement
                        </button>
                    </div>
                </section>
                {isFormShown && <VoyageOrderCreationForm />}
            </main>
        </div>
    );
}
