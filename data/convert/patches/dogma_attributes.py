def patch(entries, patches, data):
    nextAttributeID = -1

    for patch in patches:
        if patch.get("new"):
            patch["name"] = patch["new"]["name"]
            # Most attributes don't care what number they have; some do.
            # A pinned `id` is used as-is and does not consume a positional ID,
            # keeping the positional IDs of all other patches stable.
            if "id" in patch["new"]:
                nextID = patch["new"]["id"]
            else:
                nextID = nextAttributeID
                nextAttributeID -= 1
            del patch["new"]

            # Check if the name is unique.
            for entry in entries.values():
                if entry["name"] == patch["name"]:
                    raise ValueError(f"Attribute name '{patch['name']}' is not unique.")

            entries[nextID] = patch
