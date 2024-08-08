for crate in crates/*; do
  if [ -d "$crate" ]; then
    echo "Processing $crate"
    cd "$crate" || exit
    cargo rdme
    cd - || exit
  fi
done
