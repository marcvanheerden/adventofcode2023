import itertools
import random 

# Revised characters for permutations
chars = "23457TJQKA"

# Generating all possible permutations of the revised characters
permutations = set(itertools.permutations(chars, len(chars)))

# Creating rows with permutations and a random integer
rows = [''.join(p) for p in permutations]

rows = [r[0:5] + " " + str(random.randint(1, 1000)) for r in rows]


# Display the first few rows to verify
print(len(rows))
    
with open("day07_big_input.txt", 'w') as file:
    for line in rows:
        file.write(line + '\n')

