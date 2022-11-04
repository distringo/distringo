import requests

tiger_listing = requests.get('https://www2.census.gov/geo/tiger/')

print(tiger_listing.json())
