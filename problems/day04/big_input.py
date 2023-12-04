import random

def generate_advent_of_code_data(cards_count):
    data = ""
    grid = []
    for card_number in range(1, cards_count + 1):
        if card_number < (cards_count - 15):
            both_sections = random.sample(range(1, 100), 20)
            first_section = both_sections[0:9]
            second_section = random.sample(both_sections[10:20] + random.sample(range(1, 100), 15), 25)
        else:
            both_sections = random.sample(range(1, 100), 35)
            first_section = both_sections[0:9]
            second_section = both_sections[10:35]

        # Formatting each card data
        grid.append(f"Card {card_number:3}: {' '.join(f'{num:2}' for num in first_section)} | {' '.join(f'{num:2}' for num in second_section)}")


    # Write to file
    with open("day04_big_input.txt", 'w') as file:
        for line in grid:
            file.write(line + '\n')

    return data

# Generate data for 5 cards as an example
sample_data = generate_advent_of_code_data(213_000)
print(sample_data)

