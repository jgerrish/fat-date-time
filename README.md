# fat-date-time
FAT filesystem date and time library

# Introduction

This crate provides a library to parse DOS FAT filesystem dates and times.

There are two functions included now:
parse_fat_date and parse_fat_time

These two functions take u16 values representing the DOS FAT date and
time bitfields and return Date and Time structures from the time
crate.
