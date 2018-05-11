import urllib.request
from pathlib import Path

URL = "http://www.kilgarriff.co.uk/BNClists/lemma.num"

def init_lemma(path="lemma.txt", override=True):
    lemma = Path(path)
    if not lemma.is_file() or override:
        urllib.request.urlretrieve(URL, path)


def process_words(path="lemma.txt", dest_path="words.txt", override=True):
    dest_file = Path(dest_path)
    source_file = Path(path)

    if not source_file.is_file():
        print("Error: can't locate the source file")
        return

    if dest_file.is_file() and not override:
        return

    with open(path, 'r') as f:
        with open(dest_path, 'a') as df:
            lines = f.readlines()

            for line in lines:
                word = line.split(' ')[2]
                if word == "": continue
                df.write(word + '\n')

            df.close()

        f.close()


if __name__ == '__main__':
    init_lemma()
    process_words()

    print("\nDone!")