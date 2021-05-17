package com.github.norwae.deduplicate;


import com.amazonaws.services.dynamodbv2.AmazonDynamoDB;
import com.amazonaws.services.dynamodbv2.AmazonDynamoDBClient;
import com.amazonaws.services.lambda.runtime.Context;
import com.amazonaws.services.lambda.runtime.RequestHandler;
import com.amazonaws.services.lambda.runtime.events.S3Event;
import com.amazonaws.services.lambda.runtime.events.models.s3.S3EventNotification;
import com.amazonaws.services.s3.AmazonS3;
import com.amazonaws.services.s3.AmazonS3Client;

public class Deduplicate implements RequestHandler<S3Event, String> {

    private final AliasRepository alias;
    private final HashedObjectFactory hasher;

    public Deduplicate() {
        String region = System.getenv().get("AWS_REGION");
        AmazonS3 s3 = new AmazonS3Client();
        AmazonDynamoDB dynamoDB = AmazonDynamoDBClient.builder().withRegion(region).build();
        alias = new AliasRepository(dynamoDB, s3);
        hasher = new HashedObjectFactory(s3);
    }

    @Override
    public String handleRequest(S3Event s3Event, Context context) {

        s3Event.getRecords().stream()
                .map(S3EventNotification.S3EventNotificationRecord::getS3)
                .map(hasher::from)
                .forEach(alias::assign);

        return "done";
    }
}
