import { API, API_ROUTE } from './api'

let container = document.getElementById('map');

import { DB } from './db'

import * as L from 'leaflet'


console.info("Initializing the map");

let map = L.map(container, {
	center: [0.0, 0.0],
	zoom: 0,
	zoomSnap: 0.25,
	zoomDelta: 0.25,
	boxZoom: false,
	doubleClickZoom: false,
});

export type ShapefileSpec = {
	id: string,
	minZoom?: number,
	maxZoom?: number,
}


export type Shapefile = {
	id: string,
	data: string,
}

const SHAPEFILES: Array<ShapefileSpec> = [
	{
		id: "tl_2010_18157_tabblock",
		minZoom: 10.0,
		maxZoom: undefined,
	},
	{
		id: "tl_2010_18157_tabblock",
		minZoom: 10.0,
		maxZoom: undefined,
	},
];

const DISTRINGO_DB = DB("distringo", 1);

async function getShapefileOrInsert(id: string): Promise<Shapefile> {
	if (await DISTRINGO_DB.then(db => db.getKey('shapefiles', id))) {
		// Shapefile is already in the database.
		return DISTRINGO_DB.then(async db => {
			const tx = db.transaction('shapefiles', 'readonly')
			const shapefile = tx.store.get(id)
			await tx.done
			return shapefile
		})
	} else {
		// Shapefile needs to get loaded first.
		const shapefileData = await API.shapefile(id).then(data => data.text())
		return DISTRINGO_DB.then(async db => {
			// Open a transaction...
			const tx = db.transaction('shapefiles', 'readwrite')
			// ...store the shapefile data...
			await tx.store.add({ id: id, data: shapefileData })
			// ...and close the transaction.
			await tx.done
			return {id: id, data: shapefileData}
		})
	}
}

// Pre-seed the database if it is not already set up.
DB("distringo", 1).then((db) => {
	SHAPEFILES.forEach(async (shapefileSpec) => {

		const id = shapefileSpec.id

		console.log(`loading shapefile ${id}`)

		getShapefileOrInsert(id).then(shapefile => {
			console.log(`drawing shapefile ${shapefile.id}`)
			const data = JSON.parse(shapefile.data)
			L.geoJSON(data, {}).addTo(map)
			console.debug(`finished drawing shapefile ${shapefile.id}`)
		})
	})
})

console.info("Adding OSM tile set");

L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
	attribution: 'Map data &copy; <a href="https://www.openstreetmap.org/">OpenStreetMap</a> contributors, <a href="https://creativecommons.org/licenses/by-sa/2.0/">CC-BY-SA</a>'
}).addTo(map);
