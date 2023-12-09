import random

def generate_advanced_sequences(num_sequences=2_000_000, length=20, max_diff_levels=19):
    data = []

    for _ in range(num_sequences):
        # Choose the number of differentiation levels (up to max_diff_levels)
        diff_levels = random.randint(1, max_diff_levels)

        # Start with a simple linear sequence
        start = random.randint(-100, 100)
        increment = random.randint(-10, 10)
        sequence = [start + i * increment for i in range(length)]

        for level in range(2, diff_levels + 1):
            if level % 2 == 0:
                # Apply a multiplication factor for even levels
                factor = random.randint(2, 4)
                sequence = [x * factor for x in sequence]
            else:
                # Apply a quadratic transformation for odd levels
                sequence = [x + i ** 2 for i, x in enumerate(sequence)]

        data.append([str(val) for val in sequence])

    return data

# Generate advanced sequences
advanced_sequences = generate_advanced_sequences()

with open("day09_big_input.txt", 'w') as file:
    for line in advanced_sequences:
        file.write(' '.join(line) + '\n')
