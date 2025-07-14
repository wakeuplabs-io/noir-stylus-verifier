## script to compile all circuits and witnesses with the specified nargo and bb version below

TEST_VECTORS_DIR="$(dirname "$(realpath "$0")")/../test_vectors"

NARGO_VERSION=1.0.0-beta.6
BB_VERSION=0.86.0

# install noirup: curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
r=$(bash -c "nargo --version")
if  [[ $r != "nargo version = $NARGO_VERSION"* ]];
then
    bash -c "noirup -v ${NARGO_VERSION}"
fi

# use bbup every time as bb --version is broken
# install bbup: curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bash -c "bbup -v ${BB_VERSION}"

for folder in "$TEST_VECTORS_DIR"/*; do
    # To get just the directory name (e.g., "add3u64" from "/path/to/add3u64")
    dir_name=$(basename "$folder")

    if [ -d "$folder" ]; then
        echo "Processing folder: $folder"


        cd "$folder" || { echo "Failed to enter folder $folder"; continue; }

        echo "Compiling $dir_name"
        rm -fr kat && mkdir kat

        # nargo execute
        # bb prove -b ./target/$dir_name.json -w ./target/$dir_name.gz -o ./target --scheme ultra_honk --oracle_hash keccak
        # bb write_vk --oracle_hash keccak -o target -b target/$dir_name.json

        nargo execute >/dev/null 2>&1 || { echo "Error in nargo execute for $dir_name"; continue; }
        bb prove -b "./target/$dir_name.json" -w "./target/$dir_name.gz" -o ./target --scheme ultra_honk --oracle_hash keccak >/dev/null 2>&1 || { echo "Error in bb prove for $dir_name"; continue; }
        bb write_vk --oracle_hash keccak -o target -b "target/$dir_name.json" >/dev/null 2>&1 || { echo "Error in bb write_vk for $dir_name"; continue; }

        if [ -d "target" ]; then
            echo "Moving files from target to kat in $dir_name"
            mv ./target/proof ./kat/proof
            mv ./target/public_inputs ./kat/public_inputs
            mv ./target/vk ./kat/vk
        else
            echo "'target' is missing in $dir_name"
        fi

        # Return to the script directory
        cd "$script_dir" || { echo "Failed to return to script directory"; exit 1; }
    fi

done

echo "All folders processed."
