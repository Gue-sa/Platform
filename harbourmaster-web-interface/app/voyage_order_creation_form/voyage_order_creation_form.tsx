export default function VoyageOrderCreationForm() {
    return (
        <div className="voyage-order-creation-form-container">
            <form className="voyage-order-creation-form" method="GET">
                <label htmlFor="executant-input">Exécutant</label>
                <select name="executant-input"></select>

                <label htmlFor="destination-input">Destination</label>
                <select name="destination-input"></select>

                <label htmlFor="cargo-type-input">Type de cargo</label>
                <select name="cargo-type-input">
                    <option value={0}>Economique</option>
                    <option value={1}>Classique</option>
                    <option value={2}>Rapide</option>
                </select>

                <label htmlFor="speed-profile-input">Profil de vitesse</label>
                <select name="speed-profile-input"></select>

                <input type="submit" value="Valider" />
            </form>
        </div>
    );
}
