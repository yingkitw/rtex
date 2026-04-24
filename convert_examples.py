#!/usr/bin/env python3
"""Simple script to convert all LaTeX examples to PDFs"""

import os
import sys
import subprocess
from pathlib import Path

def main():
    # Find all .tex files in examples directory
    examples_dir = Path("examples")
    output_dir = Path("output")
    
    if not examples_dir.exists():
        print("Error: examples directory not found")
        sys.exit(1)
    
    # Create output directory
    output_dir.mkdir(exist_ok=True)
    
    # Find all .tex files
    tex_files = list(examples_dir.glob("*.tex"))
    
    if not tex_files:
        print("No .tex files found in examples directory")
        sys.exit(1)
    
    print(f"Found {len(tex_files)} LaTeX files to convert:")
    for tex_file in tex_files:
        print(f"  - {tex_file.name}")
    print()
    
    # Convert each file
    success_count = 0
    for tex_file in tex_files:
        print(f"Converting {tex_file.name}...")
        
        # Output PDF path
        pdf_path = output_dir / f"{tex_file.stem}.pdf"
        
        # Run latex-rs converter
        try:
            result = subprocess.run(
                ["cargo", "run", "--", str(tex_file), "-o", str(pdf_path)],
                capture_output=True,
                text=True,
                cwd=Path.cwd()
            )
            
            if result.returncode == 0 and pdf_path.exists():
                size = pdf_path.stat().st_size / 1024  # KB
                print(f"  ✓ Success: {pdf_path} ({size:.1f} KB)")
                success_count += 1
            else:
                print(f"  ✗ Failed: {result.stderr.strip()}")
                
        except Exception as e:
            print(f"  ✗ Error: {e}")
    
    print(f"\nConversion complete: {success_count}/{len(tex_files)} files converted")
    
    # List output files
    if success_count > 0:
        print("\nGenerated PDFs:")
        for pdf_file in sorted(output_dir.glob("*.pdf")):
            size = pdf_file.stat().st_size / 1024  # KB
            print(f"  - {pdf_file.name} ({size:.1f} KB)")

if __name__ == "__main__":
    main()
