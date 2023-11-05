use distringo_census_datalink::get_recursive_directory_listing;

#[tokio::main]
async fn main() {
	println!("{:?}", get_recursive_directory_listing().await);
}
