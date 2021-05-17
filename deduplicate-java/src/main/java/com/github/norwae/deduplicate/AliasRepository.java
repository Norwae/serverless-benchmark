package com.github.norwae.deduplicate;

import com.amazonaws.services.dynamodbv2.AmazonDynamoDB;
import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBMapper;
import com.amazonaws.services.s3.AmazonS3;
import com.github.norwae.deduplicate.model.CanonicalName;
import com.github.norwae.deduplicate.model.FileAlias;
import lombok.RequiredArgsConstructor;
import org.apache.commons.logging.Log;
import org.apache.commons.logging.LogFactory;

import java.util.Base64;

@RequiredArgsConstructor
public class AliasRepository {
    private final AmazonDynamoDB dynamoDB;
    private final AmazonS3 s3;
    public void assign(HashedS3Object hashedS3Object) {
        var mapper = new DynamoDBMapper(dynamoDB);
        String hashKey = Base64.getEncoder().encodeToString(hashedS3Object.getSha512());
        var canonical = mapper.load(CanonicalName.class, hashKey);

        if (canonical == null) {
            canonical = new CanonicalName(hashKey, hashedS3Object.getKey());
            mapper.save(canonical);
            System.out.println("No canonical found, put this as new canonical " + hashedS3Object);
        } else {
            System.out.println("Canonical " + canonical + " found for " + hashedS3Object + ", deleting old");
            s3.deleteObject(hashedS3Object.getBucket(), hashedS3Object.getKey());
        }

        var aliasEntry = new FileAlias(hashedS3Object.getKey(), canonical.getCanonicalName());
        mapper.save(aliasEntry);
        System.out.println("Saved alias " + aliasEntry);
    }
}
