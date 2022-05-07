const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require('copy-webpack-plugin');
const path = require('path');

module.exports = {
	entry: './index.tsx',
	mode: 'development',
	output: {
		path: path.resolve(__dirname, '../dist'),
		filename: 'bundle.js',
	},
	module: {
		rules: [
			{ test: /\.svg$/, use: 'raw-loader' },
			{ test: /\.s[ac]ss$/, use: 'sass-loader' },
			{ test: /\.tsx?$/, use: 'ts-loader' },
		],
	},
	resolve: {
		extensions: ['.tsx', '.ts', '.js'],
		alias: {
			"react": "preact/compat",
			"react-dom/test-utils": "preact/test-utils",
			"react-dom": "preact/compat",     // Must be below test-utils
			"react/jsx-runtime": "preact/jsx-runtime"
		},
	},
	plugins: [
		new HtmlWebpackPlugin({
			template: './index.html'
		}),
		new CopyPlugin({
			patterns: [
				{ from: "./assets/", to: "." },
				{ from: "./css/", to: "css" },
			],
		}),
	],
};
