# generated by ChatGPT

import random

def generate_advent_code_output_v2(num_lines, line_length, symbols, number_freq=0.05, symbol_freq=0.03, output_file='advent_code_output_v2.txt'):
    """
    Generate a grid of characters with random symbols and numbers of length 2 to 4 digits.

    :param num_lines: Number of lines in the grid.
    :param line_length: Number of characters per line.
    :param symbols: String of symbols to be randomly inserted.
    :param number_freq: Frequency of numbers in the grid.
    :param symbol_freq: Frequency of symbols in the grid.
    :param output_file: Name of the file to write the output.
    """
    grid = []

    for _ in range(num_lines):
        line = []
        while len(line) < line_length:
            rand_num = random.random()
            if rand_num < number_freq:
                # Insert a number of length 2 to 4 digits
                num_length = random.randint(2, 4)
                number = str(random.randint(10**(num_length-1), 10**num_length - 1))
                # Ensure the line length is not exceeded
                if len(line) + len(number) > line_length:
                    number = number[:line_length - len(line)]
                line.append(number)
            elif rand_num < number_freq + symbol_freq:
                # Insert a symbol
                line.append(random.choice(symbols))
            else:
                # Insert a period
                line.append('.')

        # Join the current sequence of characters
        line = ''.join(line)
        grid.append(line[0:line_length])

    # Write to file
    with open(output_file, 'w') as file:
        for line in grid:
            file.write(line + '\n')

    return output_file

# Example usage
output_file_v2 = generate_advent_code_output_v2(
    num_lines=1_400,
    line_length=1_400,
    symbols="*%#@$+-",
    output_file="day03_big_input.txt"
)

output_file_v2

