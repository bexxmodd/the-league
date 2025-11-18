.PHONY: generate-crds install-crds generate-rbac manifests

# Generate CRD YAML files from Rust code
generate-crds:
	@echo "Generating CRD YAML files..."
	@cargo run --bin generate-crds

install-crds: generate-crds
	@echo "Installing CRDs to Kubernetes cluster..."
	@kubectl apply -k Config/crds

uninstall-crds:
	@echo "Removing CRDs from Kubernetes cluster..."
	@kubectl delete -k Config/crds --ignore-not-found=true

# --- Project Variables ---
# The name of the binary that generates the CRD YAMLs
CRD_GENERATOR := generate-crds
# The name of the binary that generates the RBAC YAMLs
RBAC_GENERATOR := generate-rbac
# Directory for final CRD output, must match generate-crds.rs
CRD_DIR := config/crd
# RBAC directory
RBAC_DIR := config/rbac
# The Docker image name (adjust as needed)
IMG := theleague-controller:v1

# The 'manifests' target generates both CRDs and RBAC manifests
manifests: $(CRD_DIR) $(RBAC_DIR) generate-rbac
	@echo "--- 1. Generating CRD YAMLs from Rust structs ---"
	# Run your custom CRD generation binary
	cargo run --bin $(CRD_GENERATOR)

	@echo "--- 2. CRD YAMLs updated successfully ---"
	@echo "--- 3. RBAC manifests generated successfully ---"

# This target ensures the directory exists before the generator runs
$(CRD_DIR):
	mkdir -p $(CRD_DIR)

# This target ensures the RBAC directory exists
$(RBAC_DIR):
	mkdir -p $(RBAC_DIR)

# Generate RBAC manifests using Rust binary
# Following kube.rs security best practices: https://kube.rs/controllers/security/#access-constriction
generate-rbac: $(RBAC_DIR)
	@echo "--- Generating RBAC manifests ---"
	cargo run --bin $(RBAC_GENERATOR)
	@echo "✓ RBAC manifests generated in $(RBAC_DIR)/"

install: manifests
	kubectl apply -k config/default

uninstall:
	kubectl delete -k config/default

docker-build:
	docker build . -t $(IMG)
