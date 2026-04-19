import type { VoyageResult } from "~/types";
import { VoyageOrder } from "~/voyage_order/voyage_order";

interface VoyageOrdersProps {
    voyage_orders: VoyageResult
}

export function VoyageOrders({ voyage_orders }: VoyageOrdersProps) {
    return (
        <div className="voyage-orders-screen">
            {voyage_orders.map(([order, version, destination]) => (
                <VoyageOrder order_data={order} current_version_data={version} destination_data={destination} />
            ))}
        </div>
    );
}
