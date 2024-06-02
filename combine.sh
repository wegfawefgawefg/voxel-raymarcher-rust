#!/bin/bash

# Output file
output_file="combined.rs"

# Clear the output file if it exists
> $output_file

# Loop through all .rs files in the src directory
for file in src/*.rs; do
  # Print the filename as a comment
  echo "// filename: $(basename "$file")" >> $output_file
  # Append the content of the file
  cat "$file" >> $output_file
  # Add a newline
  echo -e "\n" >> $output_file
done

echo "All files have been combined into $output_file"
