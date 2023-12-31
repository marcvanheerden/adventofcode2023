# Generated by ChatGPT with some slight modification

import random

# Define the colors and their weights
colors = ["green", "red", "blue"]
weights = [0.4, 0.3, 0.3]

# Function to generate a single draw
def generate_draw():
    random.shuffle(colors)  # Shuffle the colors
    num_colors = random.randint(1, len(colors))  # Determine the number of colors to include
    draw = ', '.join([f"{random.randint(1, 15)} {colors[i]}" for i in range(num_colors)])
    return draw


# Function to generate a game
def generate_game(game_num):
    num_draws = random.randint(1, 10)  # You can adjust the number of draws per game
    draws = '; '.join([generate_draw() for _ in range(num_draws)])
    return f"Game {game_num}: {draws}"

# Function to generate input data for a specified number of games
def generate_input_data(num_games):
    input_data = [generate_game(game_num) for game_num in range(1, num_games + 1)]
    return '\n'.join(input_data)

# Set the number of games you want to generate
num_games_to_generate = 100_000  # Adjust as needed

# Generate the input data
input_data = generate_input_data(num_games_to_generate)

# Write the input data to a text file
with open("day02_big_input.txt", "w") as file:
    file.write(input_data)

print("Input data generated and saved to advent_of_code_input.txt")

