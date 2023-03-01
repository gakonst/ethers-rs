const semver = require("semver");

const previousVersion = (currentVersion, releaseType) => {
    const parsedVersion = semver.parse(currentVersion);

    switch (releaseType) {
        case "major": {
            return `v${parsedVersion.major - 1}.0.0`;
        }
        case "minor": {
            return `v${parsedVersion.major}.${parsedVersion.minor - 1}.0`;
        }
        case "patch": {
            return `v${parsedVersion.major}.${parsedVersion.minor}.${parsedVersion.patch - 1}`;
        }
        case "alpha": {
            return `v${parsedVersion.major}.${parsedVersion.minor}.${parsedVersion.patch}-alpha.${
                parsedVersion.prerelease[1] - 1
            }`;
        }
        case "beta": {
            return `v${parsedVersion.major}.${parsedVersion.minor}.${parsedVersion.patch}-beta.${
                parsedVersion.prerelease[1] - 1
            }`;
        }
        case "rc": {
            return `v${parsedVersion.major}.${parsedVersion.minor}.${parsedVersion.patch}-rc.${
                parsedVersion.prerelease[1] - 1
            }`;
        }
    }
};

const [currentVersion, releaseType] = process.argv.slice(-2);
console.log(previousVersion(currentVersion, releaseType));
