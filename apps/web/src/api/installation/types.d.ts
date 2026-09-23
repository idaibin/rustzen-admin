declare namespace Installation {
    type ServiceState = "healthy" | "unavailable" | "incompatible";

    interface Service {
        id: string;
        state: ServiceState;
        releaseVersion: string | null;
    }

    interface Info {
        releaseVersion: string;
        compositionId: string;
        buildId: string;
        webDigest: string;
        featureIds: string[];
        services: Service[];
        capabilities: string[];
    }
}
