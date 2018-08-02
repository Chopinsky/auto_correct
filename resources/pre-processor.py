import urllib.request
import time
from pathlib import Path

URL = "http://www.kilgarriff.co.uk/BNClists/lemma.num"
ALPHABET = "abcdefghijklmnopqrstuvwxyz"

WORD_LIMIT_HIGH = 144000
WORD_LIMIT_LOW = 72000
BASE_SCORE = 1270

def init_lemma(override=True, path="lemma.txt"):
    lemma = Path(path)
    if not lemma.is_file() or override:
        urllib.request.urlretrieve(URL, path)


def process_words(override=True, skip_score=False, path="lemma.txt", dest_path="words.txt"):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    with open(path, 'r') as f:
        with open(dest_path, 'w') as df:
            lines = f.readlines()

            for line in lines:
                word_info = line.split(' ')

                if word_info[2] == "": continue

                if skip_score:
                    word = word_info[2]
                else:
                    word = word_info[2] + "," + word_info[1]

                df.write(word + '\n')

            df.close()

        f.close()


def process_n_cut(override=True, cut_level=2, path="freq_50k.txt", dest_path="freq_50k.txt"):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    with open(path, 'r') as f:
        with open(dest_path, 'w') as df:
            count = 0
            lines = f.readlines()

            for line in lines:
                if line == "" or line == "word,count\n":
                    continue

                word_info = line.replace("\n", "").split(',')

                if word_info[0] != "" and word_info[1] != "":
                    score = int(int(word_info[1]) / BASE_SCORE)
                    df.write(word_info[0] + "," + str(score) + "\n")

                count += 1

                if cut_level == 0  and count == WORD_LIMIT_LOW:
                    break
                elif cut_level == 1 and count == WORD_LIMIT_HIGH:
                    break

            print(count)
            df.close()

        f.close()


def process_n_save_reverse(override=True, find_all=True, path="./en-us/freq_50k.txt", dest_path="./en-us/freq_50k_precalc_rev.txt"):
    source_file = Path(path)
    dest_file = Path(dest_path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    dict = set()
    with open(path, 'r') as f:
        with open(dest_path, 'w') as df:
            print("Info: process begins...\n")

            count = 0
            lines = f.readlines()

            for line in lines:
                if line == "" or line == "word,count\n":
                    continue

                word_info = line.replace("\n", "").split(',')

                if word_info[0] != "" and word_info[1] != "" and word_info[0] not in dict:
                    dict.add(word_info[0])

            ts = time.gmtime()
            print(time.strftime("%H:%M:%S", ts), len(dict), "\n")

            rev_dict = {}
            for word in dict:
                neighbors = words_in_one_edit(word, dict, find_all)
                if len(neighbors) == 0:
                    continue
                
                for neighbor in neighbors:
                    if neighbor in rev_dict:
                        rev_dict[neighbor] = rev_dict[neighbor] + ";" + word
                    else:
                        rev_dict[neighbor] = word

                count += 1
                if count % 500 == 0:
                    ts = time.gmtime()
                    print(time.strftime("%H:%M:%S", ts), " - ", count, "\n")

            for (rev_word, linked) in rev_dict.items():
                result = rev_word + "^" + linked + "\n"
                df.write(result)

            df.close()

        f.close()

    return dict


def process_n_save(override=True, find_all=True, path="./en-us/freq_50k.txt", dest_path="./en-us/freq_50k_precalc.txt"):
    source_file = Path(path)
    dest_file = Path(dest_path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    dict = set()
    with open(path, 'r') as f:
        with open(dest_path, 'w') as df:
            print("Info: process begins...\n")

            count = 0
            lines = f.readlines()

            for line in lines:
                if line == "" or line == "word,count\n":
                    continue

                word_info = line.replace("\n", "").split(',')

                if word_info[0] != "" and word_info[1] != "" and word_info[0] not in dict:
                    dict.add(word_info[0])

            ts = time.gmtime()
            print(time.strftime("%H:%M:%S", ts), len(dict), "\n")

            for word in dict:
                neighbors = words_in_one_edit(word, dict, find_all)
                if len(neighbors) == 0:
                    continue
                
                result = word + "^"
                for neighbor in neighbors:
                    if len(neighbor) > 0:
                        result = result + neighbor + ";"

                result = result + "\n"
                df.write(result)

                count += 1
                if count % 500 == 0:
                    ts = time.gmtime()
                    print(time.strftime("%H:%M:%S", ts), " - ", count, "\n")

            df.close()

        f.close()

    return dict


def words_in_one_edit(source, dict, find_all):
    result = set()
    length = len(source)

    for i in range(length+1):
        if length > 1:
            # delete...
            delete = source[0:i] + source[i+1:]
            check_and_add(delete, source, dict, result, find_all)

            # swap...
            if i+2 <= length:
                swap = source[0:i] + source[i+1] + source[i] + source[i+2:]
                check_and_add(swap, source, dict, result, find_all)

        for letter in ALPHABET:
            # insert...
            insert = source[0:i] + letter + source[i:]
            check_and_add(insert, source, dict, result, find_all)

            # replace...
            replace = source[0:i] + letter + source[i+1:]
            check_and_add(replace, source, dict, result, find_all)

    return result


def check_and_add(target, source, dict, result, find_all):
    if target != source and target not in result:
        if find_all or target in dict:
            result.add(target)


if __name__ == '__main__':
    '''Standard words processing
    init_lemma(False)
    process_words(dest_path="words2.txt")
    '''

    '''Processing with different level of dictionary entry numbers
    process_n_cut(cut_level=0, dest_path="uniq_low.txt")
    '''

    #dict = process_n_save()
    process_n_save_reverse()

    print("\nDone...\n")