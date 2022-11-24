import pickle
import requests
import itertools
import os
from tqdm import tqdm
from bs4 import BeautifulSoup

def load_pagecache():
    try:
        with open('pagecache.pkl', 'rb') as f:
            return pickle.load(f)
    except FileNotFoundError:
        return {}
    
def save_pagecache(cache):
    with open('pagecache.pkl', 'wb') as f:
        pickle.dump(cache, f)

PAGES = list(range(1, 156))
PAGECACHE = load_pagecache()

def make_svg_url(id):
    id = id.split('-')[0]
    return f"https://dddfont.com/m/{id}/{id}_0.svg"

def make_page_url(id):
    return f"https://dddfont.com/page/{id}/"

print("PAGES", len(PAGES))
print("PAGECACHE", len(PAGECACHE))

s = requests.Session()

for page in tqdm(PAGES, desc="Discovering pages"):
    if page in PAGECACHE:
        # print("Skipping page", page)
        continue
    tqdm.write(f"PAGE {page}", end='')
    r = s.get(make_page_url(page))
    r.raise_for_status()

    soup = BeautifulSoup(r.text, 'html.parser')
    links = soup.find('ul', id='entry')

    page_ids = []
    for link in links.find_all('a'):
        href = link['href']
        id = href.split('/')[-2]
        page_ids.append(id)

    tqdm.write(f" -> {len(page_ids)} ids")
    
    PAGECACHE[page] = page_ids
    save_pagecache(PAGECACHE)

SVG_IDS = list(itertools.chain(*PAGECACHE.values()))

os.makedirs('svg', exist_ok=True)

for svg_id in tqdm(SVG_IDS, desc="Downloading SVGs"):
    path = os.path.join('svg', f'{svg_id}.svg')
    if os.path.exists(path):
        continue

    svg_url = make_svg_url(svg_id)
    r = s.get(svg_url)
    r.raise_for_status()
    with open(path, 'w') as f:
        f.write(r.text)

