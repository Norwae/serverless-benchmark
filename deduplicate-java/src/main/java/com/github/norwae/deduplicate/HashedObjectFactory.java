package com.github.norwae.deduplicate;

import com.amazonaws.services.lambda.runtime.events.models.s3.S3EventNotification;
import lombok.RequiredArgsConstructor;
import lombok.SneakyThrows;

import com.amazonaws.services.s3.AmazonS3;
import org.apache.commons.logging.Log;
import org.apache.commons.logging.LogFactory;

import java.security.MessageDigest;

@RequiredArgsConstructor
public class HashedObjectFactory {
    private final AmazonS3 client;

    @SneakyThrows
    public HashedS3Object from(S3EventNotification.S3Entity s3Entity) {
        var sha512 = MessageDigest.getInstance("SHA-512");
        System.out.println("Retrieving target object " + s3Entity);
        var response = client.getObject(s3Entity.getBucket().getName(), s3Entity.getObject().getKey());
        var stream = response.getObjectContent();

        System.out.println("Retrieved object " + response);

        var buffer = new byte[4096];
        int read;
        while ((read = stream.read(buffer)) != -1) {
            sha512.update(buffer, 0, read);
        }
        var digested = sha512.digest();
        return new HashedS3Object(s3Entity.getBucket().getName(), s3Entity.getObject().getKey(), digested);
    }
}
