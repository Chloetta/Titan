#!/bin/bash

# Limit the loop to 10 iterations for testing purposes
for i in {1..10}; do
  # Generate a random IP address
  ip=$((RANDOM % 256)).$((RANDOM % 256)).$((RANDOM % 256)).$((RANDOM % 256))
  
  # Print the generated IP address
  echo "Current IP Address: $ip"
  
  # Sleep for 5 seconds before generating a new IP address
  sleep 5
done
