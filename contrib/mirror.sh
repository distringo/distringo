#!/bin/sh

year="$1"

case "$year" in

	"2010")
		# wget -m "ftp://ftp2.census.gov/geo/tiger/TIGER2010/"
		wget -m "ftp://ftp2.census.gov/programs-surveys/decennial/2010/data/01-Redistricting_File--PL_94-171/Wisconsin"
		# wget -m "ftp://ftp2.census.gov/programs-surveys/decennial/2010/data/01-Redistricting_File--PL_94-171"
		;;

	"2020")
		wget -m "ftp://ftp2.census.gov/geo/tiger/TIGER2020PL/STATE/"
		wget -m "ftp://ftp2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171"
		# wget -m "https://www2.census.gov/programs-surveys/decennial/2020/data/01-Redistricting_File--PL_94-171/"
		;;

esac
