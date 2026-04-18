import { Metrics } from "~/metrics/metrics";
import type { Route } from "./+types/home";
import { VoyageOrders } from "~/orders/orders";
import { Map } from "~/map/map";
import VoyageOrderCreationForm from "~/voyage_order_creation_form/voyage_order_creation_form";
import { useState } from "react";

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

    return (
        <div className="harbourmaster-interface">
            <header>
                <h1 className="main-title">Interface Armateur</h1>
            </header>
            <main>
                <section className="screen-section left-screen-section">
                    <article className="map-article">
                        <p className="article-title map-article-title">Carte</p>
                        <Map />
                    </article>
                    <article className="metrics-aarticle">
                        <p className="article-title metrics-article-title">
                            Données
                        </p>
                        <Metrics />
                    </article>
                </section>
                <section className="screen-section right-screen-section">
                    <article className="voyage-orders-article">
                        <p className="article-title voyage-orders-article-title">
                            Ordres de Voyage
                        </p>
                        <VoyageOrders />
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
