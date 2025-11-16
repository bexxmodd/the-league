.PHONY: generate-crds install-crds

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

