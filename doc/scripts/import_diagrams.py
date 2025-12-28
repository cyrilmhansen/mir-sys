import os
import glob
import re

DOXYGEN_OUT = "_build/doxygen-html"
OUTPUT_RST = "generated_diagrams.rst"

def clean_dot_content(content):
    # Remove local URL references that won't work in Sphinx
    content = re.sub(r'URL="[^"]*"', '', content)
    content = re.sub(r'tooltip="[^"]*"', '', content)
    # Fix leftover commas from removed attributes.
    content = re.sub(r',\s*,', ',', content)
    content = re.sub(r',\s*];', '];', content)
    content = re.sub(r',\s*]', ']', content)
    content = re.sub(r',\s*;', ';', content)
    return content

def generate_title(filename):
    name = os.path.basename(filename).replace(".dot", "")
    name = name.replace("_coll__graph", " Collaboration Graph")
    name = name.replace("_dep", " Dependencies")
    name = name.replace("struct", "")
    name = name.replace("__", "_")
    name = name.replace("_", " ")
    return name.strip()

def main():
    if not os.path.exists(DOXYGEN_OUT):
        print(f"Warning: {DOXYGEN_OUT} not found. run 'make doxygen' first.")
        return

    dot_files = glob.glob(os.path.join(DOXYGEN_OUT, "**", "*.dot"), recursive=True)
    print(f"DEBUG: Found {len(dot_files)} dot files in {DOXYGEN_OUT}")
    
    with open(OUTPUT_RST, "w") as f:
        f.write("Generated Doxygen Diagrams\n")
        f.write("--------------------------\n\n")
        
        if not dot_files:
            f.write("No diagrams found.\n")
            return

        for dot_file in sorted(dot_files):
            # Skip legend
            if os.path.basename(dot_file).startswith("graph_legend"):
                continue
                
            title = generate_title(dot_file)
            
            with open(dot_file, "r") as df:
                content = df.read()
                content = clean_dot_content(content)
            
            f.write(f"**{title}**\n\n")
            f.write(".. graphviz::\n\n")
            # Indent content
            for line in content.splitlines():
                f.write(f"   {line}\n")
            f.write("\n")
            
    print(f"Generated {OUTPUT_RST} with {len(dot_files)} diagrams.")

if __name__ == "__main__":
    main()
