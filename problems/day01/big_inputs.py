import random
import string

# List of numbers spelled out
numbers_spelled = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "1", "2", "7"]

def insert_spelled_numbers(s):
    """Insert spelled-out numbers at random positions in the string"""
    for number in numbers_spelled:
        if random.choice([True, False]):  # Randomly decide whether to insert a number
            insert_pos = random.randint(0, len(s))
            s = s[:insert_pos] + number + s[insert_pos:]
    return s

def generate_random_string(length):
    """Generate a random string of given length"""
    chars = string.ascii_lowercase + string.digits
    # Generate the rest of the string
    rand_string = ''.join(random.choice(chars) for _ in range(length - 1))
    # Insert spelled-out numbers
    return insert_spelled_numbers(rand_string)

# Example usage
with open('day01_big_input.txt', "w") as file:
    for _ in range(100000):  
        output = generate_random_string(random.randint(2, 500))
        file.write(output + "\n")

