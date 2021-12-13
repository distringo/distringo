import { API, API_ROUTE } from "./api";

let container = document.getElementById("map");

import { DB } from "./db";

import * as L from "leaflet";
import { GeoJSON } from "geojson";

console.info("Initializing the map");

let map = L.map(container, {
	center: [38.0, -97.0],
	zoom: 5,
	zoomSnap: 0.25,
	zoomDelta: 0.25,
	boxZoom: false,
	doubleClickZoom: false,
});

export type ShapefileSpec = {
	id: string;
	minZoom?: number;
	maxZoom?: number;
	boundingBox?: L.LatLngBounds;
};

export type Shapefile = {
	id: string;
	data: GeoJSON.FeatureCollection;
};

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

function getStartingMapBounds(shapefiles: Array<ShapefileSpec>): {
	minZoom?: number;
	maxZoom?: number;
	bounds?: L.LatLngBounds;
} {
	let minZoom = undefined;
	let maxZoom = undefined;

	let bounds: L.LatLngBounds = L.latLngBounds([]);

	for (let shapefile of SHAPEFILES) {
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

	console.debug(bounds);

	return { minZoom, maxZoom, bounds };
}

let { minZoom, maxZoom, bounds } = getStartingMapBounds(SHAPEFILES);

map.setMaxZoom(maxZoom);
map.setMinZoom(minZoom);
map.fitBounds(bounds);

const DISTRINGO_DB = DB("distringo", 1);

async function getShapefileOrInsert(id: string): Promise<Shapefile> {
	const db = await DISTRINGO_DB;

	if ((await db.getKey("shapefiles", id)) !== undefined) {
		// Shapefile is already in the database.
		//
		// Open a readonly transaction...
		const tx = db.transaction("shapefiles", "readonly");
		// ...get the value from the IDB...
		const shapefile: Shapefile = await tx.store.get(id);
		// ...and then close out the transaction.
		await tx.done;
		return shapefile;
	} else {
		// Shapefile needs to get loaded first.
		//
		// Load up the data first.
		const shapefileData = await API.shapefile(id).then((data) =>
			data.json()
		);

		// Then, open a read-write transaction...
		const tx = db.transaction("shapefiles", "readwrite");
		// ...add the value into the IDB...
		await tx.store.put({ id: id, data: shapefileData });
		// ...then fetch the value back out...
		const shapefile: Shapefile = await tx.store.get(id);
		// ...and then close out the transaction.
		await tx.done;
		return shapefile;
	}
}

// Pre-seed the database if it is not already set up.
SHAPEFILES.forEach((shapefileSpec) => {
	const id = shapefileSpec.id;

	console.log(`loading shapefile ${id}`);

	getShapefileOrInsert(id).then((shapefile) => {
		console.log(`drawing shapefile ${shapefile.id}`);
		const data: GeoJSON.FeatureCollection = shapefile.data;
		let layer = L.geoJSON(data, {
			style: {
				color: "rgba(0.25, 0.25, 0.25, 0.60)",
			},
		});
		layer.addTo(map);
		console.debug(`finished drawing shapefile ${shapefile.id}`);
	});
});

console.info("Adding OSM tile set");

L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
	attribution:
		'Map data &copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors, <a href="https://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>',
}).addTo(map);
