#!/bin/bash

#
# Shared Script - GCP Artifact Registry Docker Image Cleaner
#
# RULE
# Delete all unnecessary images such that for each image name there are at most 2-3 images stored at all times, 
# thus allowing rolling back at least 2-3 versions if needed.
#
# USAGE
# Run clear_stale_artifact_images.sh <ARG_DRY_RUN> <ARG_REPOSITORY_ADDRESS>
# Examples: clear_stale_artifact_images.sh 1 us-na1-docker.pkg.dev/DummyProject-420/my-ar (for a dry-run)
#           clear_stale_artifact_images.sh 0 us-na1-docker.pkg.dev/DummyProject-420/my-ar (for an actual run)
#

# Input Arguments
dry_run=$1 # 0=false (submit deletion requests), 1=true (don't perform any deletions)
repository=$2 # e.g.: us-na1-docker.pkg.dev/DummyProject-420/my-ar

# Variables
image_limit_per_service=2 # How many images to keep for each service (by name)

artifact_processed_lines=() # Will store lines of data retrieved from the AR (name digest created_at updated_at)

if [[ $# -ne 2 ]] ; then
    echo "Usage:"
    echo "clear_stale_artifact_images.sh 1 us-na1-docker.pkg.dev/DummyProject-420/my-ar (for a dry-run)"
    echo "clear_stale_artifact_images.sh 0 us-na1-docker.pkg.dev/DummyProject-420/my-ar (for an actual run)"
    exit 1
fi

# Announce purpose of run (dry-run or actual run)
if (( dry_run == 1 )); then
    echo "Note that this is a dry-run and that no resources will be deleted."
else
    echo "Note that this is an actual workflow execution and that resources *may* be deleted."
fi

# Retrieve the list of all published images and their metadata from the Google Artifact Registry
artifact_registry_result_lines=$(gcloud artifacts docker images list $repository --sort-by=~UPDATE_TIME)

# Build an array of strings which contain artifact metadata, 1 artifact per string
current_header_count=0
while read line; do

    # Skip over header lines
    if (( current_header_count < 1 )); then
        current_header_count=$(( current_header_count+1 ))
        continue
    fi

    #
    # Now, each line of text contains data about a published Docker image (i.e the 4 returned data fields per row)
    #

    # Tokenise the line by whitespace to separate the Docker image data on the input line
    read -ra image_metadata -d '' <<< "$line"

    # Capture the specific metadata values
    image_name=${image_metadata[0]}
    digest=${image_metadata[1]}
    created_at=${image_metadata[2]}
    updated_at=${image_metadata[3]}

    # And append to the auxiliary array
    artifact_processed_lines+=("$image_name $digest $created_at $updated_at")

done <<< "$artifact_registry_result_lines"

# For every artifact, find all its other siblings
# and delete them if they are old and if they amount to more than the image limit per artifact (i.e count_tagged_images_for_service >= 3)
current_artifact_image_count=0
current_image_name=""
deleted_count=0
deleted=()
for (( i=0; i<${#artifact_processed_lines[@]}; i++ )); do

    current_artifact="${artifact_processed_lines[$i]}"
    current_artifact_image_count=1

    read -ra image_metadata -d '' <<< "$current_artifact"

    # Capture the specific metadata values
    current_image_name=${image_metadata[0]}
    current_digest=${image_metadata[1]}
    current_created_at=${image_metadata[2]}
    current_updated_at=${image_metadata[3]}

    # Skip over artifacts which had their siblings cleared up already (1)
    deleted_already=$(echo ${deleted[@]} | grep -ow "$current_image_name" | wc -w)
    if (( deleted_already > 0 )); then
        continue
    fi

    for (( j=i+1; j<${#artifact_processed_lines[@]}; j++ )); do

        moving_artifact="${artifact_processed_lines[$j]}"

        read -ra image_metadata -d '' <<< "$moving_artifact"

        # Capture the specific metadata values
        moving_image_name=${image_metadata[0]}
        moving_digest=${image_metadata[1]}
        moving_created_at=${image_metadata[2]}
        moving_updated_at=${image_metadata[3]}

        # Skip over artifacts which had their siblings cleared up already (2)
        deleted_already=$(echo ${deleted[@]} | grep -ow "$moving_image_name" | wc -w)
        if (( deleted_already > 0 )); then
            continue
        fi

        # Found an image with the same name
        if [[ "$current_image_name" == "$moving_image_name" ]]; then
            
            current_artifact_image_count=$(( current_artifact_image_count + 1 ))

            # If the count for this kind of image is greater than the threshold
            if (( current_artifact_image_count > image_limit_per_service )); then
                # Then delete the current published image given its ID (name & digest hash)
                image_id="$moving_image_name@$moving_digest"
                if (( dry_run == 0 )); then
                    # Only send delete requests if NOT in a dry-run
                    res=$(gcloud artifacts docker images delete $image_id --async --quiet --delete-tags)
                fi
                deleted_count=$(( deleted_count + 1 ))
                echo "Marked for deletion: $moving_digest"
            fi
        fi
    done

    # Record as deleted in order to skip future iterations handling the same image by name
    deleted+=($current_image_name)
done

# Announce workflow status at script completion time
echo "Submitted a total of $deleted_count redundant Docker images for deletion."
if (( dry_run == 0 )); then
    echo "Check the operation status in Artifact Registry (https://console.cloud.google.com/artifacts?cloudshell=true&project=aztebot-403621)."
else
    echo "Dry-run complete. No resources were deleted in GCP."
fi