package net.hydrius.pacts.spring;

import org.springframework.amqp.core.*;
import org.springframework.amqp.rabbit.connection.CachingConnectionFactory;
import org.springframework.amqp.rabbit.connection.ConnectionFactory;
import org.springframework.amqp.rabbit.core.RabbitAdmin;
import org.springframework.amqp.rabbit.core.RabbitTemplate;
import org.springframework.amqp.rabbit.listener.SimpleMessageListenerContainer;
import org.springframework.amqp.rabbit.listener.adapter.MessageListenerAdapter;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.testcontainers.containers.RabbitMQContainer;

import static net.hydrius.pacts.spring.SpringBootMessageTest.*;

@Configuration
public class RabbitConfig {

    @Bean
    public RabbitMQContainer rabbitMQContainer() {
        RabbitMQContainer container = new RabbitMQContainer("rabbitmq:3.13.1");
        container.start();
        return container;
    }

    @Bean
    public ConnectionFactory connectionFactory(RabbitMQContainer rabbitMQContainer) {
        CachingConnectionFactory factory = new CachingConnectionFactory(
                rabbitMQContainer.getHost(),
                rabbitMQContainer.getMappedPort(5672)
        );
        factory.setUsername(rabbitMQContainer.getAdminUsername());
        factory.setPassword(rabbitMQContainer.getAdminPassword());

        return factory;
    }

    @Bean
    public AmqpAdmin amqpAdmin(ConnectionFactory cf) {
        return new RabbitAdmin(cf);
    }

    @Bean
    public RabbitTemplate rabbitTemplate(ConnectionFactory cf) {
        return new RabbitTemplate(cf);
    }

    @Bean
    TopicExchange exchange() {
        return new TopicExchange(EXCHANGE, true, false);

    }

    @Bean
    Queue queue() {
        return new Queue(QUEUE, true);
    }

    @Bean
    Binding binding(Queue q, TopicExchange ex) {
        return BindingBuilder.bind(q).to(ex).with(ROUTING_KEY);
    }

    @Bean
    public SimpleMessageListenerContainer messageListenerContainer(ConnectionFactory cf, MessageListenerAdapter listenerAdapter) {
        SimpleMessageListenerContainer container = new SimpleMessageListenerContainer();
        container.setConnectionFactory(cf);
        container.setQueueNames(QUEUE);
        container.setMessageListener(listenerAdapter);
        return container;
    }

    @Bean
    public MessageListenerAdapter listenerAdapter(TestListener receiver) {
        return new MessageListenerAdapter(receiver, "handleMessage");
    }

}