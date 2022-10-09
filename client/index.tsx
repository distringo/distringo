// import { API, API_ROUTE } from "./api";

let container = document.querySelector("main.content");

// import { DB } from "./db";

import * as L from "leaflet";
// import { GeoJSON } from "geojson";

import * as React from "preact";
import * as RL from "react-leaflet";
import "preact/debug";

// type SessionId = string

const SHAPEFILES: Array<ShapefileSpec> = [
	{
		id: "tl_2010_18157_tabblock",
		minZoom: 10.0,
		maxZoom: undefined,
		boundingBox: L.latLngBounds(
			[40.214365, -87.095357],
			[40.562959, -86.694597]
		),
	},
	// {
	// 	id: "tl_2010_18157_tabblock",
	// 	minZoom: 10.0,
	// 	maxZoom: undefined,
	// },
];

let { minZoom, maxZoom, bounds } = getStartingMapBounds(SHAPEFILES);

interface ShapefileSpec {
	id: string;
	minZoom?: number;
	maxZoom?: number;
	boundingBox?: L.LatLngBounds;
}

function getStartingMapBounds(shapefiles: Array<ShapefileSpec>): {
	minZoom?: number;
	maxZoom?: number;
	bounds?: L.LatLngBounds;
} {
	let minZoom = undefined;
	let maxZoom = undefined;

	const bounds: L.LatLngBounds = L.latLngBounds([]);

	for (let shapefile of shapefiles) {
		if (shapefile.minZoom < minZoom || minZoom === undefined) {
			minZoom = shapefile.minZoom;
		}

		if (shapefile.maxZoom > maxZoom || maxZoom === undefined) {
			maxZoom = shapefile.maxZoom;
		}

		if (shapefile.boundingBox) {
			bounds.extend(shapefile.boundingBox);
		}
	}

	return { minZoom, maxZoom, bounds };
}

interface Session {
	name: string;
	id: string;
}

interface SessionPickerProps {
	sessions: Session[];
	onSessionSelectionChanged: (id: string) => void;
}

class SessionPickerComponent extends React.Component<SessionPickerProps> {
	constructor(props: SessionPickerProps) {
		super(props);
	}

	onSessionSelected = (event: Event) => {
		event.preventDefault();

		if (event.currentTarget instanceof HTMLSelectElement) {
			let value: string = event.currentTarget.value;

			if (value.length > 0) {
				this.props.onSessionSelectionChanged(value);
			}
		} else {
			throw new Error(
				"onSessionSelected called when current target was not a Select element."
			);
		}
	};

	render() {
		return (
			<div class="session-picker">
				<h1>Pick a Session</h1>
				<select onChange={this.onSessionSelected}>
					<option selected={true} value={null} disabled={true}>
						Pick an Option
					</option>
					{this.props.sessions.map((session: Session) => (
						<option key={session.id} value={session.id}>
							{session.name}
						</option>
					))}
				</select>
			</div>
		);
	}
}

interface MapControlProps {}

interface MapControlState {}

class MapControl extends React.Component<MapControlProps, MapControlState> {
	constructor(props: MapControlProps) {
		super(props);
	}

	render() {
		return <RL.MapContainer></RL.MapContainer>;
	}
}

interface DistringoClientProps {
	apiUrl: URL;
	initialSession: string | null;
}

interface DistringoClientState {
	session: string | null;
}

class DistringoClient extends React.Component<
	DistringoClientProps,
	DistringoClientState
> {
	constructor(props: DistringoClientProps) {
		super(props);
		this.state = { session: props.initialSession };
	}

	componentDidMount() {
		return;
	}

	componentWillUnmount() {
		return;
	}

	shouldComponentUpdate(
		_nextProps: DistringoClientProps,
		_nextState: DistringoClientState
	): boolean {
		return true;
	}

	getSnapshotBeforeUpdate(
		_prevProps: DistringoClientProps,
		_prevState: DistringoClientState
	) {
		return;
	}

	componentDidUpdate(
		_prevProps: DistringoClientProps,
		_prevState: DistringoClientState,
		_snapshot: ReturnType<typeof this.getSnapshotBeforeUpdate> | undefined
	) {
		return;
	}

	onSessionSelected = (id: string) => {
		this.setState({ session: id });
	};

	render() {
		// TODO(rye): Fill this out with real data.
		const sessions: Session[] = [
			{ id: "asdf", name: "asdf" },
			{ id: "jkl;", name: "jkl;" },
		];

		// TODO(rye): If only one session, automate the pick to just the available session.
		if (this.state.session === null) {
			return (
				<SessionPickerComponent
					sessions={sessions}
					onSessionSelectionChanged={this.onSessionSelected}
				/>
			);
		} else {
			// TODO(rye): Extract this out.
			return (
				<RL.MapContainer
					className="distringo-map-container"
					center={[38.0, -97.0]}
					zoom={5}
					zoomSnap={0.25}
					zoomDelta={0.25}
					boxZoom={false}
					doubleClickZoom={false}
				>
					<RL.TileLayer
						attribution='Map data &copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors, <a href="https://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>'
						url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
					/>
				</RL.MapContainer>
			);
		}
	}
}

const App = (
	<DistringoClient
		apiUrl={new URL(window.location.href)}
		initialSession={null}
	/>
);

React.render(App, container);

// console.info("Initializing the map");

// export type Shapefile = {
// 	id: string;
// 	data: GeoJSON.FeatureCollection;
// };

// map.setMaxZoom(maxZoom);
// map.setMinZoom(minZoom);
// map.fitBounds(bounds);

// const DISTRINGO_DB = DB("distringo", 1);

// async function getShapefileOrInsert(id: string): Promise<Shapefile> {
// 	const db = await DISTRINGO_DB;

// 	if ((await db.getKey("shapefiles", id)) !== undefined) {
// 		// Shapefile is already in the database.
// 		//
// 		// Open a readonly transaction...
// 		const tx = db.transaction("shapefiles", "readonly");
// 		// ...get the value from the IDB...
// 		const shapefile: Shapefile = await tx.store.get(id);
// 		// ...and then close out the transaction.
// 		await tx.done;
// 		return shapefile;
// 	} else {
// 		// Shapefile needs to get loaded first.
// 		//
// 		// Load up the data first.
// 		const shapefileData = await API.shapefile(id).then((data) =>
// 			data.json()
// 		);

// 		// Then, open a read-write transaction...
// 		const tx = db.transaction("shapefiles", "readwrite");
// 		// ...add the value into the IDB...
// 		await tx.store.put({ id: id, data: shapefileData });
// 		// ...then fetch the value back out...
// 		const shapefile: Shapefile = await tx.store.get(id);
// 		// ...and then close out the transaction.
// 		await tx.done;
// 		return shapefile;
// 	}
// }

// // Pre-seed the database if it is not already set up.
// SHAPEFILES.forEach((shapefileSpec) => {
// 	const id = shapefileSpec.id;

// 	console.log(`loading shapefile ${id}`);

// 	getShapefileOrInsert(id).then((shapefile) => {
// 		console.log(`drawing shapefile ${shapefile.id}`);
// 		const data: GeoJSON.FeatureCollection = shapefile.data;
// 		let layer = L.geoJSON(data, {
// 			style: {
// 				color: "rgba(0.25, 0.25, 0.25, 0.60)",
// 			},
// 		});
// 		layer.addTo(map);
// 		console.debug(`finished drawing shapefile ${shapefile.id}`);
// 	});
// });
