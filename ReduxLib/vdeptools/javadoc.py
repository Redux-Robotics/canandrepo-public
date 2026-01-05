"""
Extracts code snippets from javadocs, stuffs them into a robot project, and checks if they compile.
"""
import bs4
from pathlib import Path
#import pprint
import functools
import dataclasses
import typing
import textwrap
import shutil
import os

@dataclasses.dataclass
class ClassName:
    package: str  
    name: str

    @classmethod
    def from_fname(cls, fname: str): # -> typing.Self:
        spl_fname = fname[:-len(".html")].split("/")
        return cls(".".join(spl_fname[:-1]), spl_fname[-1])
    
    @property
    def fqcn(self):
        return self.package + "." + self.name

@dataclasses.dataclass
class DocEntry:
    # fully qualified class name
    class_name: ClassName
    # entry id
    id: str
    # entry_text
    section: bs4.Tag

    @classmethod
    def f(cls, fname, id, section):
        return cls(ClassName.from_fname(fname), id, section)

def sanitize_name(s: str) -> str:
    v = []
    for c in s:
        if c.isalnum() or c in "$_":
            v.append(c)
        elif c == ".":
            v.append("_1")
        else:
            v.append("_")
    return "".join(v)

def get_soup(fname: Path) -> bs4.BeautifulSoup:
    with open(fname, "r") as f:
        return bs4.BeautifulSoup(f.read(), 'lxml')

def fetch_class_pages(jdoc_root: Path) -> typing.List[bs4.Tag]:
    """Extracts a list of class pages from the javadoc class index"""
    soup = get_soup(jdoc_root/"allclasses-index.html")
    pages = set()

    all_a = functools.reduce(lambda x, y: x + y, map(lambda x: x.find_all('a'), soup.find("div", id="all-classes-table").find_all("div", class_="col-first")))
    for a in all_a:
        pages.add(a['href'])
    return list(pages)

def extract_doc_entries(jdoc_root: Path, fname: str) -> typing.List[DocEntry]:
    #<section class="class-description" id="class-description"> -- id = <description>
    #<section class="constructor-details" id="constructor-detail">
    #<section class="detail" id="&lt;init&gt;()">

    #<section class="method-details" id="method-detail">
    #<section class="detail" id="handleMessage(com.reduxrobotics.canand.CanandMessage)">
    #<div class="block"> (don't bother)

    ents = []

    soup = get_soup(jdoc_root/fname)
    ents.append(DocEntry.f(fname, "<class>", soup.find("section", id="class-description")))

    if ctor_detail := soup.find("section", id="constructor-detail"):
        for ctor_section in ctor_detail.find_all("section", class_="detail"):
            ents.append(DocEntry.f(fname, ctor_section['id'], ctor_section))
    
    if method_detail := soup.find("section", id="method-detail"):
        for method_section in method_detail.find_all("section", class_="detail"):
            ents.append(DocEntry.f(fname, method_section['id'], method_section))
    
    return ents

def generate_source_file(jdoc_root: Path, page: str):
    """
    Generates source files from <pre> tags. 

    Directives may be added to the <pre> tags' class= attribute.
    For example, to exclude a <pre> tag from code, use the no-code class.

    To include additional java packages, use something like:

    <pre class="include-com_reduxrobotics_frames_FrameData"> [code here] </pre>

    Trailing underscores imply a wildcard import:
    <pre class="include-com_reduxrobotics_frames_*"> [code here] </pre>
    """
    cname = ClassName.from_fname(page)
    doc_entries = extract_doc_entries(jdoc_root, page)
    snippets: typing.List[str] = []
    imports = {f"import {cname.package}.*;"}
    snippet_class = sanitize_name(cname.name)
    for ent in doc_entries:
        matches: typing.Iterable[bs4.Tag] = ent.section.find_all("pre")
        if not matches:
            continue
        for idx, pre in enumerate(matches):
            # the <pre> tag is allowed to have class attributes.
            # we abuse this to embed directives for the source file generator.
            attrs = pre.get('class', [])
            if "not-code" in attrs:
                # this is not a code block. continue.
                continue
            
            for attr in attrs:
                if attr.startswith("include-"):
                    ipt = attr[len("include-"):].replace("_", ".")
                    if ipt.endswith("."):
                        imports.add(f"import {ipt}*;")
                    else:
                        imports.add(f"import {ipt};")

            snippets.append(f"""
    /** From {snippet_class} */
    public static void {sanitize_name(snippet_class + "_" + ent.id)}_{idx}() {{
{textwrap.indent(pre.get_text(), " " * 8)}
    }}""")

    if not snippets:
        return None, None
    newline = "\n"

    return f"{snippet_class}Snippets", f"""
package snippets;
{newline.join(imports)}

class {snippet_class}Snippets {{
{newline.join(snippets)}
}}
"""

def main(jdoc_root: Path):
    pages = fetch_class_pages(jdoc_root)

    snippets_proj_root = Path(os.path.dirname(os.path.realpath(__file__)))/"Snippets-java"
    snippets_root = snippets_proj_root/"src/main/java/snippets"
    shutil.rmtree(snippets_root/"", ignore_errors=True)
    snippets_root.mkdir()

    for pg in pages:
        #doc_entries = extract_doc_entries(jdoc_root, pg)
        name, src = generate_source_file(jdoc_root, pg)
        if not name:
            continue
        with open(snippets_root/f"{name}.java", "w") as f:
            f.write(src)
            print(src)

if __name__ == "__main__":
    import sys
    #jdoc_root = Path("../ReduxLib/build/docs/javadoc")
    jdoc_root = Path(sys.argv[1])
    main(jdoc_root)
